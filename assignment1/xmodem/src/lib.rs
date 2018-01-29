use std::io;

#[cfg(test)] mod tests;
mod read_ext;

use read_ext::ReadExt;

const SOH: u8 = 0x01;
const EOT: u8 = 0x04;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;
const CAN: u8 = 0x18;

pub struct Xmodem<R> {
    packet: u8,
    inner: R,
    started: bool
}

impl Xmodem<()> {
    /// Transmits `data` to the receiver `to` using the XMODEM protocol. If the
    /// length of the total data yielded by `data` is not a multiple of 128
    /// bytes, the data is padded with zeroes and sent to the receiver.
    ///
    /// Returns the number of bytes written to `to`, excluding padding zeroes.
    pub fn transmit<R, W>(mut data: R, to: W) -> io::Result<usize>
        where W: io::Read + io::Write, R: io::Read
    {
        let mut transmitter = Xmodem::new(to);
        let mut packet = [0u8; 128];
        let mut written = 0;
        'next_packet: loop {
            let n = data.read_max(&mut packet)?;
            packet[n..].iter_mut().for_each(|b| *b = 0);

            if n == 0 {
                transmitter.write_packet(&[])?;
                return Ok(written);
            }

//            println!("+++++++++++++++++++++++");
            for _ in 0..10 {
                match transmitter.write_packet(&packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(n) => {
                        written += n;
                        continue 'next_packet;
                    }
                }
            }

            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "bad transmit"));
        }
    }

    /// Receives `data` from `from` using the XMODEM protocol and writes it into
    /// `into`. Returns the number of bytes read from `from`, a multiple of 128.
    pub fn receive<R, W>(from: R, mut into: W) -> io::Result<usize>
       where R: io::Read + io::Write, W: io::Write
    {
        let mut receiver = Xmodem::new(from);
        let mut packet = [0u8; 128];
        let mut received = 0;
        'next_packet: loop {
            for _ in 0..10 {
                match receiver.read_packet(&mut packet) {
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                    Ok(0) => break 'next_packet,
                    Ok(n) => {
                        received += n;
                        into.write_all(&packet)?;
                        continue 'next_packet;
                    }
                }
            }

            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "bad receive"));
        }

        Ok(received)
    }
}

impl<T: io::Read + io::Write> Xmodem<T> {
    /// Returns a new `Xmodem` instance with the internal reader/writer set to
    /// `inner`. The returned instance can be used for both receiving
    /// (downloading) and sending (uploading).
    pub fn new(inner: T) -> Self {
        Xmodem { packet: 1, started: false, inner: inner }
    }

    /// Reads a single byte from the inner I/O stream. If the byte is `CAN`, a
    /// `ConnectionAborted` error is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails or if the read
    /// byte was `CAN`.
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.inner.read_exact(&mut buf)?;
        match buf[0] {
            CAN => Err(io::Error::new(io::ErrorKind::ConnectionAborted, "received CAN")),
            byte => Ok(byte)
        }
    }

    /// Writes a single byte to the inner I/O stream.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the inner stream fails.
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.inner.write_all(&[byte])
    }

    /// Reads a single byte from the inner I/O stream and compares it to `byte`.
    /// If they differ, a `CAN` byte is written out to the inner stream and an
    /// error of `InvalidData` with the message `expected` is returned.
    /// Otherwise the byte is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails, if the read
    /// byte was not `byte`, or if writing the `CAN` byte failed on byte
    /// mismatch.
    fn expect_byte_or_cancel(&mut self, byte: u8, msg: &'static str) -> io::Result<u8> {
        match self.expect_byte(byte, msg) {
            Err(e) => {self.write_byte(CAN)?; Err(e)},
            Ok(byte) => Ok(byte)
        }
    }

    /// Reads a single byte from the inner I/O stream and compares it to `byte`.
    /// If they differ, an error of `InvalidData` with the message `expected` is
    /// returned. Otherwise the byte is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the inner stream fails or if the read
    /// byte was not `byte`.
    fn expect_byte(&mut self, byte: u8, expected: &'static str) -> io::Result<u8> {
        match self.read_byte() {
            Ok(b) if b == byte => Ok(byte),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, expected)),
        }
    }

    /// Reads (downloads) a single packet from the inner stream using the XMODEM
    /// protocol. On success, returns the number of bytes read (always 128).
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing to the inner stream fails at any
    /// point. Also returns an error if the XMODEM protocol indicates an error.
    /// In particular, an `InvalidData` error is returned when:
    ///
    ///   * The sender's first byte for a packet isn't `EOT` or `SOH`.
    ///   * The sender doesn't send a second `EOT` after the first.
    ///   * The received packet numbers don't match the expected values.
    ///
    /// An error of kind `Interrupted` is returned if a packet checksum fails.
    ///
    /// An error of kind `UnexpectedEof` is returned if `buf.len() < 128`.
    pub fn read_packet(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 128 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Invalid buffer length"));
        }
        if !self.started {
            self.write_byte(NAK)?;
            self.started = true;
        }
//        println!("enter read_packet");
        match self.read_byte() {
            Ok(SOH) => {
                let mut recved = 0;
                let packet = self.packet;
                if let Err(e) = self.expect_byte_or_cancel(packet, "packet number mismatch") {
                    return Err(e);
                }
                if let Err(e) = self.expect_byte_or_cancel(255 - packet, "1s completement packet number mismatch") {
                    return Err(e);
                }
                let mut checksum = 0u16;
                for i in 0..buf.len() {
                    if let Ok(b) = self.read_byte() {
                        buf[i] = b;
                    } else {
                        buf[i] = CAN;
                    }
                    checksum += (buf[i] as u16) % 256;
                    recved += 1;
//                    println!("{}: got byte {}, checksum {}", i, buf[i], checksum);
                }
                let checksum = (checksum % 256) as u8;
//                println!("expecting checksum {}", checksum);
                match self.expect_byte(checksum, "checksum mismatch") {
                    Ok(_) => {
//                        println!("sending ack");
                        self.write_byte(ACK)?;
                        self.packet += 1;
                        Ok(recved)
                    }
                    _ => {
//                        println!("sending nack");
                        self.write_byte(NAK)?;
                        Err(io::Error::new(io::ErrorKind::Interrupted, 
                                               "checksum mismatch"))
                    }
                }
            }
            Ok(EOT) => {
//                println!("got EOT");
                self.write_byte(NAK)?;
                if let Err(e) = self.expect_byte(EOT, "expect EOT") {
                    return Err(e);
                }
                self.write_byte(ACK)?;
                self.started = false;
                Ok(0)
            }
            b => return Err(io::Error::new(io::ErrorKind::InvalidData, 
                                           format!("Neither SOH nor EOT received, got {:?}", b)))
        }
    }

    /// Sends (uploads) a single packet to the inner stream using the XMODEM
    /// protocol. If `buf` is empty, end of transmissions is sent. Users of this
    /// interface should ensure that `write_packet(&[])` is called when data
    /// transmission is complete. On success, returns the number of bytes
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or writing to the inner stream fails at any
    /// point. Also returns an error if the XMODEM protocol indicates an error.
    /// In particular, an `InvalidData` error is returned when:
    ///
    ///   * The receiver's first byte isn't a `NAK`.
    ///   * The receiver doesn't respond with a `NAK` to the first `EOT`.
    ///   * The receiver doesn't respond with an `ACK` to the second `EOT`.
    ///   * The receiver responds to a complete packet with something besides
    ///     `ACK` or `NAK`.
    ///
    /// An error of kind `UnexpectedEof` is returned if `buf.len() < 128 &&
    /// buf.len() != 0`.
    ///
    /// An error of kind `Interrupted` is returned if a packet checksum fails.
    pub fn write_packet(&mut self, buf: &[u8]) -> io::Result<usize> {
        match buf.len() {
            0 => {
//                println!("sending EOT");
                self.write_byte(EOT)?;
                if let Err(e) = self.expect_byte(NAK, "Expect NAK") {
                    return Err(e);
                }
                self.write_byte(EOT)?;
                if let Err(e) = self.expect_byte(ACK, "Expect ACK") {
                    return Err(e);
                }
                self.started = false;
                Ok(0)
            } 
            128 => {
                if !self.started {
                    if let Err(e) = self.expect_byte(NAK, "Expect NAK") {
                        return Err(e);
                    }
                    self.started = true;
                }
                let packet = self.packet;
                let mut sent = 0;

//                println!("sending SOH");
                self.write_byte(SOH)?;
                self.write_byte(packet)?;
                self.write_byte(255 - packet)?;

                let mut checksum = 0u16;
                for i in 0..buf.len() {
                    sent += 1;
                    checksum += (buf[i] as u16) % 256;
                    self.write_byte(buf[i])?;
//                    println!("{} send byte {}, checksum {}", i, buf[i], checksum);
                }
                let checksum = (checksum % 256) as u8;
//                println!("start sending checksum {}", checksum);
                self.write_byte(checksum)?;
//                println!("finished sending checksum");

                match self.read_byte() {
                    Ok(ACK) => {self.packet += 1; return Ok(sent)},
                    Ok(NAK) => Err(io::Error::new(io::ErrorKind::Interrupted, 
                                           "checksum mismatch")),
                    Err(e) => Err(e),
                    Ok(_) => Err(io::Error::new(io::ErrorKind::InvalidData, 
                                                       "Neither ACK nor NACK received"))
                }
            }
            _ => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Invalid buffer length"))
        }
    }

    /// Flush this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// Errors
    ///
    /// It is considered an error if not all bytes could be written due to I/O
    /// errors or EOF being reached.
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
