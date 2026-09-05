//! CI-friendly Modbus TCP lab path (localhost loopback only).
//!
//! Minimal MBAP + PDU framing for the water_heater map — **no** `tokio-modbus`,
//! serial RTU, TLS, or hardware. Speaks enough of:
//! - FC01 Read Coils
//! - FC03 Read Holding Registers
//! - FC05 Write Single Coil
//! - FC06 Write Single Register
//!
//! Writes that hit a mapped address update the [`ModbusBridge`] HomeCooked
//! backend (same foreign→HC path as the in-memory mock).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::adapter::ModbusBridge;
use super::map::RegisterKind;
use super::mock::ModbusSlave;
use crate::backend::MemoryBackend;
use crate::bridge::{Bridge, ForeignRaw, ForeignRef};
use crate::error::Error;

/// Shared bridge behind the lab TCP accept loop.
pub type SharedModbusBridge = Arc<Mutex<ModbusBridge<MemoryBackend>>>;

/// Result of [`spawn_modbus_tcp_lab`].
pub type SpawnedModbusTcp = (SocketAddr, SharedModbusBridge, JoinHandle<()>);

/// Wrap a bridge for concurrent Modbus TCP handlers.
pub fn shared_bridge(bridge: ModbusBridge<MemoryBackend>) -> SharedModbusBridge {
    Arc::new(Mutex::new(bridge))
}

/// Bind `127.0.0.1:0` (or another addr) and serve the water_heater (or any)
/// in-memory map over Modbus TCP.
pub fn spawn_modbus_tcp_lab(
    addr: impl ToSocketAddrs,
    bridge: SharedModbusBridge,
) -> Result<SpawnedModbusTcp, Error> {
    let listener = TcpListener::bind(addr)?;
    let local = listener.local_addr()?;
    let shared = Arc::clone(&bridge);
    let join = thread::spawn(move || accept_loop(listener, shared));
    Ok((local, bridge, join))
}

fn accept_loop(listener: TcpListener, bridge: SharedModbusBridge) {
    // Idle accept should not hang tests forever if the listener is dropped.
    let _ = listener.set_nonblocking(false);
    while let Ok((stream, _)) = listener.accept() {
        let bridge = Arc::clone(&bridge);
        thread::spawn(move || {
            let _ = serve_connection(stream, &bridge);
        });
    }
}

fn serve_connection(stream: TcpStream, bridge: &SharedModbusBridge) -> Result<(), Error> {
    stream.set_nodelay(true)?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let mut stream = stream;
    loop {
        let adu = match read_adu(&mut stream) {
            Ok(adu) => adu,
            Err(Error::Io(ref msg))
                if msg.contains("early eof") || msg.contains("UnexpectedEof") =>
            {
                return Ok(());
            }
            Err(Error::Io(ref msg))
                if msg.contains("timed out")
                    || msg.contains("WouldBlock")
                    || msg.contains("Resource temporarily unavailable") =>
            {
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        let response = {
            let mut guard = bridge
                .lock()
                .map_err(|_| Error::Backend("modbus tcp lock".into()))?;
            handle_adu(&mut guard, &adu)
        };
        stream.write_all(&response)?;
    }
}

/// Thin sync Modbus TCP client for lab / CI tests.
#[derive(Debug)]
pub struct ModbusTcpClient {
    stream: TcpStream,
    unit_id: u8,
    next_tid: u16,
}

impl ModbusTcpClient {
    pub fn connect(addr: SocketAddr, unit_id: u8) -> Result<Self, Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
        Ok(Self {
            stream,
            unit_id,
            next_tid: 1,
        })
    }

    pub fn read_holding_registers(&mut self, start: u16, quantity: u16) -> Result<Vec<u16>, Error> {
        let mut pdu = Vec::with_capacity(5);
        pdu.push(0x03);
        pdu.extend_from_slice(&start.to_be_bytes());
        pdu.extend_from_slice(&quantity.to_be_bytes());
        let resp = self.transact(&pdu)?;
        parse_read_holding_response(&resp, quantity)
    }

    pub fn write_single_register(&mut self, address: u16, value: u16) -> Result<(), Error> {
        let mut pdu = Vec::with_capacity(5);
        pdu.push(0x06);
        pdu.extend_from_slice(&address.to_be_bytes());
        pdu.extend_from_slice(&value.to_be_bytes());
        let resp = self.transact(&pdu)?;
        if resp.len() != 5 || resp[0] != 0x06 {
            return Err(modbus_err(format!("unexpected FC06 response: {resp:?}")));
        }
        Ok(())
    }

    pub fn read_coils(&mut self, start: u16, quantity: u16) -> Result<Vec<bool>, Error> {
        let mut pdu = Vec::with_capacity(5);
        pdu.push(0x01);
        pdu.extend_from_slice(&start.to_be_bytes());
        pdu.extend_from_slice(&quantity.to_be_bytes());
        let resp = self.transact(&pdu)?;
        parse_read_coils_response(&resp, quantity)
    }

    pub fn write_single_coil(&mut self, address: u16, value: bool) -> Result<(), Error> {
        let mut pdu = Vec::with_capacity(5);
        pdu.push(0x05);
        pdu.extend_from_slice(&address.to_be_bytes());
        let coil_val: u16 = if value { 0xFF00 } else { 0x0000 };
        pdu.extend_from_slice(&coil_val.to_be_bytes());
        let resp = self.transact(&pdu)?;
        if resp.len() != 5 || resp[0] != 0x05 {
            return Err(modbus_err(format!("unexpected FC05 response: {resp:?}")));
        }
        Ok(())
    }

    fn transact(&mut self, pdu: &[u8]) -> Result<Vec<u8>, Error> {
        let tid = self.next_tid;
        self.next_tid = self.next_tid.wrapping_add(1);
        let adu = build_adu(tid, self.unit_id, pdu);
        self.stream.write_all(&adu)?;
        let resp_adu = read_adu(&mut self.stream)?;
        let (resp_tid, unit, resp_pdu) = parse_adu(&resp_adu)?;
        if resp_tid != tid {
            return Err(modbus_err(format!(
                "tid mismatch: sent {tid}, got {resp_tid}"
            )));
        }
        if unit != self.unit_id {
            return Err(modbus_err(format!(
                "unit mismatch: expected {}, got {unit}",
                self.unit_id
            )));
        }
        if resp_pdu.is_empty() {
            return Err(modbus_err("empty PDU"));
        }
        if resp_pdu[0] & 0x80 != 0 {
            let code = resp_pdu.get(1).copied().unwrap_or(0);
            return Err(modbus_err(format!(
                "exception FC={:#04x} code={code}",
                resp_pdu[0]
            )));
        }
        Ok(resp_pdu)
    }
}

fn modbus_err(detail: impl Into<String>) -> Error {
    Error::InvalidRaw {
        detail: detail.into(),
    }
}

fn build_adu(tid: u16, unit_id: u8, pdu: &[u8]) -> Vec<u8> {
    let len = (pdu.len() + 1) as u16;
    let mut out = Vec::with_capacity(6 + 1 + pdu.len());
    out.extend_from_slice(&tid.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes()); // protocol id
    out.extend_from_slice(&len.to_be_bytes());
    out.push(unit_id);
    out.extend_from_slice(pdu);
    out
}

fn read_adu(stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
    let mut header = [0u8; 6];
    read_exact_eof(stream, &mut header)?;
    let len = u16::from_be_bytes([header[4], header[5]]) as usize;
    if len == 0 || len > 260 {
        return Err(modbus_err(format!("invalid MBAP length {len}")));
    }
    let mut rest = vec![0u8; len];
    read_exact_eof(stream, &mut rest)?;
    let mut adu = Vec::with_capacity(6 + len);
    adu.extend_from_slice(&header);
    adu.extend_from_slice(&rest);
    Ok(adu)
}

fn read_exact_eof(stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), Error> {
    let mut read = 0;
    while read < buf.len() {
        match stream.read(&mut buf[read..]) {
            Ok(0) => {
                return Err(Error::Io("early eof".into()));
            }
            Ok(n) => read += n,
            Err(e) => return Err(Error::from(e)),
        }
    }
    Ok(())
}

fn parse_adu(adu: &[u8]) -> Result<(u16, u8, Vec<u8>), Error> {
    if adu.len() < 8 {
        return Err(modbus_err(format!("ADU too short ({})", adu.len())));
    }
    let tid = u16::from_be_bytes([adu[0], adu[1]]);
    let proto = u16::from_be_bytes([adu[2], adu[3]]);
    if proto != 0 {
        return Err(modbus_err(format!("bad protocol id {proto}")));
    }
    let len = u16::from_be_bytes([adu[4], adu[5]]) as usize;
    if adu.len() != 6 + len {
        return Err(modbus_err(format!(
            "ADU length mismatch: header {len}, bytes {}",
            adu.len() - 6
        )));
    }
    let unit = adu[6];
    let pdu = adu[7..].to_vec();
    Ok((tid, unit, pdu))
}

fn handle_adu(bridge: &mut ModbusBridge<MemoryBackend>, adu: &[u8]) -> Vec<u8> {
    let (tid, unit, pdu) = match parse_adu(adu) {
        Ok(v) => v,
        Err(_) => return build_adu(0, 0, &exception_pdu(0x00, 0x03)),
    };
    let expected = bridge.slave().slave_id;
    if unit != expected {
        return build_adu(
            tid,
            unit,
            &exception_pdu(pdu.first().copied().unwrap_or(0), 0x0B),
        );
    }
    let resp_pdu = handle_pdu(bridge, &pdu);
    build_adu(tid, unit, &resp_pdu)
}

fn handle_pdu(bridge: &mut ModbusBridge<MemoryBackend>, pdu: &[u8]) -> Vec<u8> {
    if pdu.is_empty() {
        return exception_pdu(0x00, 0x01);
    }
    match pdu[0] {
        0x01 => fc01_read_coils(bridge.slave(), pdu),
        0x03 => fc03_read_holding(bridge.slave(), pdu),
        0x05 => fc05_write_coil(bridge, pdu),
        0x06 => fc06_write_holding(bridge, pdu),
        fc => exception_pdu(fc, 0x01),
    }
}

fn exception_pdu(fc: u8, code: u8) -> Vec<u8> {
    vec![fc | 0x80, code]
}

fn fc03_read_holding(slave: &ModbusSlave, pdu: &[u8]) -> Vec<u8> {
    if pdu.len() != 5 {
        return exception_pdu(0x03, 0x03);
    }
    let start = u16::from_be_bytes([pdu[1], pdu[2]]);
    let qty = u16::from_be_bytes([pdu[3], pdu[4]]);
    if qty == 0 || qty > 125 {
        return exception_pdu(0x03, 0x03);
    }
    let mut out = Vec::with_capacity(2 + 2 * qty as usize);
    out.push(0x03);
    out.push((qty * 2) as u8);
    for i in 0..qty {
        let addr = start.wrapping_add(i);
        let v = slave.get_holding(addr);
        out.extend_from_slice(&v.to_be_bytes());
    }
    out
}

fn fc01_read_coils(slave: &ModbusSlave, pdu: &[u8]) -> Vec<u8> {
    if pdu.len() != 5 {
        return exception_pdu(0x01, 0x03);
    }
    let start = u16::from_be_bytes([pdu[1], pdu[2]]);
    let qty = u16::from_be_bytes([pdu[3], pdu[4]]);
    if qty == 0 || qty > 2000 {
        return exception_pdu(0x01, 0x03);
    }
    let byte_count = ((qty as usize) + 7) / 8;
    let mut out = Vec::with_capacity(2 + byte_count);
    out.push(0x01);
    out.push(byte_count as u8);
    let mut bytes = vec![0u8; byte_count];
    for i in 0..qty {
        if slave.get_coil(start.wrapping_add(i)) {
            let idx = i as usize;
            bytes[idx / 8] |= 1 << (idx % 8);
        }
    }
    out.extend_from_slice(&bytes);
    out
}

fn fc06_write_holding(bridge: &mut ModbusBridge<MemoryBackend>, pdu: &[u8]) -> Vec<u8> {
    if pdu.len() != 5 {
        return exception_pdu(0x06, 0x03);
    }
    let address = u16::from_be_bytes([pdu[1], pdu[2]]);
    let value = u16::from_be_bytes([pdu[3], pdu[4]]);
    if let Err(resp) = apply_holding_write(bridge, address, value) {
        return resp;
    }
    // Echo request
    pdu.to_vec()
}

fn apply_holding_write(
    bridge: &mut ModbusBridge<MemoryBackend>,
    address: u16,
    value: u16,
) -> Result<(), Vec<u8>> {
    let mapped = bridge
        .map()
        .entry_for_address(RegisterKind::Holding, address)
        .is_some();
    if mapped {
        let device_id = bridge.map().device_id.clone();
        let foreign =
            ForeignRef::holding(&device_id, address).map_err(|_| exception_pdu(0x06, 0x04))?;
        bridge
            .write_foreign(&foreign, ForeignRaw::Register(value))
            .map_err(|_| exception_pdu(0x06, 0x04))?;
    } else {
        bridge.slave_mut().set_holding(address, value);
    }
    Ok(())
}

fn fc05_write_coil(bridge: &mut ModbusBridge<MemoryBackend>, pdu: &[u8]) -> Vec<u8> {
    if pdu.len() != 5 {
        return exception_pdu(0x05, 0x03);
    }
    let address = u16::from_be_bytes([pdu[1], pdu[2]]);
    let raw = u16::from_be_bytes([pdu[3], pdu[4]]);
    let on = match raw {
        0xFF00 => true,
        0x0000 => false,
        _ => return exception_pdu(0x05, 0x03),
    };
    if let Err(resp) = apply_coil_write(bridge, address, on) {
        return resp;
    }
    pdu.to_vec()
}

fn apply_coil_write(
    bridge: &mut ModbusBridge<MemoryBackend>,
    address: u16,
    on: bool,
) -> Result<(), Vec<u8>> {
    let mapped = bridge
        .map()
        .entry_for_address(RegisterKind::Coil, address)
        .is_some();
    if mapped {
        let device_id = bridge.map().device_id.clone();
        let foreign =
            ForeignRef::coil(&device_id, address).map_err(|_| exception_pdu(0x05, 0x04))?;
        bridge
            .write_foreign(&foreign, ForeignRaw::Coil(on))
            .map_err(|_| exception_pdu(0x05, 0x04))?;
    } else {
        bridge.slave_mut().set_coil(address, on);
    }
    Ok(())
}

fn parse_read_holding_response(pdu: &[u8], quantity: u16) -> Result<Vec<u16>, Error> {
    if pdu.len() < 2 || pdu[0] != 0x03 {
        return Err(modbus_err(format!("bad FC03 response: {pdu:?}")));
    }
    let byte_count = pdu[1] as usize;
    if byte_count != quantity as usize * 2 || pdu.len() != 2 + byte_count {
        return Err(modbus_err(format!(
            "FC03 length mismatch: qty={quantity} bytes={byte_count} len={}",
            pdu.len()
        )));
    }
    let mut out = Vec::with_capacity(quantity as usize);
    for chunk in pdu[2..].chunks_exact(2) {
        out.push(u16::from_be_bytes([chunk[0], chunk[1]]));
    }
    Ok(out)
}

fn parse_read_coils_response(pdu: &[u8], quantity: u16) -> Result<Vec<bool>, Error> {
    if pdu.len() < 2 || pdu[0] != 0x01 {
        return Err(modbus_err(format!("bad FC01 response: {pdu:?}")));
    }
    let byte_count = pdu[1] as usize;
    let expected = ((quantity as usize) + 7) / 8;
    if byte_count != expected || pdu.len() != 2 + byte_count {
        return Err(modbus_err(format!(
            "FC01 length mismatch: qty={quantity} bytes={byte_count}"
        )));
    }
    let mut out = Vec::with_capacity(quantity as usize);
    for i in 0..quantity as usize {
        let bit = (pdu[2 + i / 8] >> (i % 8)) & 1;
        out.push(bit != 0);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::{Bridge, PointRef};
    use homecooked_schema::Value;

    #[test]
    fn build_and_parse_adu_roundtrip() {
        let pdu = vec![0x03, 0x00, 0x00, 0x00, 0x02];
        let adu = build_adu(7, 1, &pdu);
        let (tid, unit, got) = parse_adu(&adu).unwrap();
        assert_eq!(tid, 7);
        assert_eq!(unit, 1);
        assert_eq!(got, pdu);
    }

    #[test]
    fn pdu_fc03_reads_seeded_holdings() {
        let mut bridge = ModbusBridge::water_heater_example().unwrap();
        let pdu = vec![0x03, 0x00, 0x00, 0x00, 0x02];
        let resp = handle_pdu(&mut bridge, &pdu);
        assert_eq!(resp[0], 0x03);
        assert_eq!(resp[1], 4);
        assert_eq!(&resp[2..6], &[0x02, 0x26, 0x01, 0xE0]); // 550, 480
    }

    #[test]
    fn pdu_fc06_updates_homecooked_backend() {
        let mut bridge = ModbusBridge::water_heater_example().unwrap();
        let pdu = vec![0x06, 0x00, 0x00, 0x02, 0x58]; // 600 tenths = 60.0
        let resp = handle_pdu(&mut bridge, &pdu);
        assert_eq!(resp, pdu);
        assert_eq!(
            bridge
                .backend()
                .get_value("water-heater-plant", "trait.temperature.setpoint_c"),
            Some(&Value::F32(60.0))
        );
        assert_eq!(
            bridge
                .read_point(
                    &PointRef::new("water-heater-plant", "trait.temperature.setpoint_c").unwrap()
                )
                .unwrap(),
            Value::F32(60.0)
        );
    }
}
