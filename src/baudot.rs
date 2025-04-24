use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::{
    fugit::MicrosDurationU32,
    gpio::{FunctionSioInput, FunctionSioOutput, Pin, PinId, PullDown},
};
use LineState::*;

const WRITE_BUF_LENGTH: usize = 1024;
const BAUD_RATE: MicrosDurationU32 = MicrosDurationU32::millis(22);
const STOP_BIT_DURATION: MicrosDurationU32 = MicrosDurationU32::millis(32);

const ASCII_TO_BAUDOT: [BaudotChar; 256] = [
    BaudotChar(BaudotShift::Keep, 0),       // null
    BaudotChar(BaudotShift::Keep, 0),       // start of heading
    BaudotChar(BaudotShift::Keep, 0),       // start of text
    BaudotChar(BaudotShift::Keep, 0),       // end of text
    BaudotChar(BaudotShift::Keep, 0),       // end of transmision
    BaudotChar(BaudotShift::FIGS, 0b10010), // enquiry
    BaudotChar(BaudotShift::Keep, 0),       // acknowledge
    BaudotChar(BaudotShift::FIGS, 0b11010), // bell
    BaudotChar(BaudotShift::Keep, 0),       // backspace
    BaudotChar(BaudotShift::Keep, 0),       // tab
    BaudotChar(BaudotShift::Keep, 0b00010), // new line
    BaudotChar(BaudotShift::Keep, 0),       // vertical tab
    BaudotChar(BaudotShift::Keep, 0),       // form feed
    BaudotChar(BaudotShift::Keep, 0b01000), // carriage return
    BaudotChar(BaudotShift::Keep, 0),       // shift out
    BaudotChar(BaudotShift::Keep, 0),       // shift in
    BaudotChar(BaudotShift::Keep, 0),       // data link escape
    BaudotChar(BaudotShift::Keep, 0),       // device control 1
    BaudotChar(BaudotShift::Keep, 0),       // device control 2
    BaudotChar(BaudotShift::Keep, 0),       // device control 3
    BaudotChar(BaudotShift::Keep, 0),       // device control 4
    BaudotChar(BaudotShift::Keep, 0),       // negative acknowledge
    BaudotChar(BaudotShift::Keep, 0),       // synchronous idle
    BaudotChar(BaudotShift::Keep, 0),       // end of transmission
    BaudotChar(BaudotShift::Keep, 0),       // cancel
    BaudotChar(BaudotShift::Keep, 0),       // end of medium
    BaudotChar(BaudotShift::Keep, 0),       // substitute
    BaudotChar(BaudotShift::Keep, 0),       // escape
    BaudotChar(BaudotShift::Keep, 0),       // file separator
    BaudotChar(BaudotShift::Keep, 0),       // group separator
    BaudotChar(BaudotShift::Keep, 0),       // record separator
    BaudotChar(BaudotShift::Keep, 0),       // unit separator
    BaudotChar(BaudotShift::Keep, 0),       // space
    BaudotChar(BaudotShift::Keep, 0),       // !
    BaudotChar(BaudotShift::FIGS, 0b10001), // "
    BaudotChar(BaudotShift::FIGS, 0b00101), // #
    BaudotChar(BaudotShift::FIGS, 0b10010), // $
    BaudotChar(BaudotShift::Keep, 0),       // %
    BaudotChar(BaudotShift::FIGS, 0b01011), // &
    BaudotChar(BaudotShift::FIGS, 0b11010), // '
    BaudotChar(BaudotShift::FIGS, 0b11110), // (
    BaudotChar(BaudotShift::FIGS, 0b01001), // )
    BaudotChar(BaudotShift::Keep, 0),       // *
    BaudotChar(BaudotShift::FIGS, 0b10001), // +
    BaudotChar(BaudotShift::FIGS, 0b00110), // ,
    BaudotChar(BaudotShift::FIGS, 0b11000), // -
    BaudotChar(BaudotShift::FIGS, 0b00111), // .
    BaudotChar(BaudotShift::FIGS, 0b10111), // /
    BaudotChar(BaudotShift::FIGS, 0b01101), // 0
    BaudotChar(BaudotShift::FIGS, 0b11101), // 1
    BaudotChar(BaudotShift::FIGS, 0b11001), // 2
    BaudotChar(BaudotShift::FIGS, 0b10000), // 3
    BaudotChar(BaudotShift::FIGS, 0b01010), // 4
    BaudotChar(BaudotShift::FIGS, 0b00001), // 5
    BaudotChar(BaudotShift::FIGS, 0b10101), // 6
    BaudotChar(BaudotShift::FIGS, 0b11100), // 7
    BaudotChar(BaudotShift::FIGS, 0b01100), // 8
    BaudotChar(BaudotShift::FIGS, 0b00011), // 9
    BaudotChar(BaudotShift::FIGS, 0b01110), // :
    BaudotChar(BaudotShift::FIGS, 0b01111), // ;
    BaudotChar(BaudotShift::Keep, 0),       // <
    BaudotChar(BaudotShift::FIGS, 0b01111), // =
    BaudotChar(BaudotShift::Keep, 0),       // >
    BaudotChar(BaudotShift::FIGS, 0b10011), // ?
    BaudotChar(BaudotShift::Keep, 0),       // @
    BaudotChar(BaudotShift::LTRS, 0b11000), // A
    BaudotChar(BaudotShift::LTRS, 0b10011), // B
    BaudotChar(BaudotShift::LTRS, 0b01110), // C
    BaudotChar(BaudotShift::LTRS, 0b10010), // D
    BaudotChar(BaudotShift::LTRS, 0b10000), // E
    BaudotChar(BaudotShift::LTRS, 0b10110), // F
    BaudotChar(BaudotShift::LTRS, 0b01011), // G
    BaudotChar(BaudotShift::LTRS, 0b00101), // H
    BaudotChar(BaudotShift::LTRS, 0b01100), // I
    BaudotChar(BaudotShift::LTRS, 0b11010), // J
    BaudotChar(BaudotShift::LTRS, 0b11110), // K
    BaudotChar(BaudotShift::LTRS, 0b01001), // L
    BaudotChar(BaudotShift::LTRS, 0b00111), // M
    BaudotChar(BaudotShift::LTRS, 0b00110), // N
    BaudotChar(BaudotShift::LTRS, 0b00011), // O
    BaudotChar(BaudotShift::LTRS, 0b01101), // P
    BaudotChar(BaudotShift::LTRS, 0b11101), // Q
    BaudotChar(BaudotShift::LTRS, 0b01010), // R
    BaudotChar(BaudotShift::LTRS, 0b10100), // S
    BaudotChar(BaudotShift::LTRS, 0b00001), // T
    BaudotChar(BaudotShift::LTRS, 0b11100), // U
    BaudotChar(BaudotShift::LTRS, 0b01111), // V
    BaudotChar(BaudotShift::LTRS, 0b11001), // W
    BaudotChar(BaudotShift::LTRS, 0b10111), // X
    BaudotChar(BaudotShift::LTRS, 0b10101), // Y
    BaudotChar(BaudotShift::LTRS, 0b10001), // Z
    BaudotChar(BaudotShift::Keep, 0),       // [
    BaudotChar(BaudotShift::Keep, 0),       // \
    BaudotChar(BaudotShift::Keep, 0),       // ]
    BaudotChar(BaudotShift::Keep, 0),       // ^
    BaudotChar(BaudotShift::FIGS, 0b11000), // _
    BaudotChar(BaudotShift::Keep, 0),       // `
    BaudotChar(BaudotShift::LTRS, 0b11000), // a
    BaudotChar(BaudotShift::LTRS, 0b10011), // b
    BaudotChar(BaudotShift::LTRS, 0b01110), // c
    BaudotChar(BaudotShift::LTRS, 0b10010), // d
    BaudotChar(BaudotShift::LTRS, 0b10000), // e
    BaudotChar(BaudotShift::LTRS, 0b10110), // f
    BaudotChar(BaudotShift::LTRS, 0b01011), // g
    BaudotChar(BaudotShift::LTRS, 0b00101), // h
    BaudotChar(BaudotShift::LTRS, 0b01100), // i
    BaudotChar(BaudotShift::LTRS, 0b11010), // j
    BaudotChar(BaudotShift::LTRS, 0b11110), // k
    BaudotChar(BaudotShift::LTRS, 0b01001), // l
    BaudotChar(BaudotShift::LTRS, 0b00111), // m
    BaudotChar(BaudotShift::LTRS, 0b00110), // n
    BaudotChar(BaudotShift::LTRS, 0b00011), // o
    BaudotChar(BaudotShift::LTRS, 0b01101), // p
    BaudotChar(BaudotShift::LTRS, 0b11101), // q
    BaudotChar(BaudotShift::LTRS, 0b01010), // r
    BaudotChar(BaudotShift::LTRS, 0b10100), // s
    BaudotChar(BaudotShift::LTRS, 0b00001), // t
    BaudotChar(BaudotShift::LTRS, 0b11100), // u
    BaudotChar(BaudotShift::LTRS, 0b01111), // v
    BaudotChar(BaudotShift::LTRS, 0b11001), // w
    BaudotChar(BaudotShift::LTRS, 0b10111), // x
    BaudotChar(BaudotShift::LTRS, 0b10101), // y
    BaudotChar(BaudotShift::LTRS, 0b10001), // z
    BaudotChar(BaudotShift::Keep, 0),       // {
    BaudotChar(BaudotShift::Keep, 0),       // |
    BaudotChar(BaudotShift::Keep, 0),       // }
    BaudotChar(BaudotShift::Keep, 0),       // ~
    BaudotChar(BaudotShift::LTRS, 0b11111), // delete
    BaudotChar(BaudotShift::Keep, 0),       // --- rest is unused ---
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
    BaudotChar(BaudotShift::Keep, 0),
];

// baudot code as defined here: https://en.wikipedia.org/wiki/Baudot_code#ITA_2_and_US-TTY
const BAUDOT_TO_ASCII: [[(u8, BaudotShift); 2]; 32] = [
    [(b'\0', BaudotShift::LTRS), (b'\0', BaudotShift::FIGS)],
    [(b'E', BaudotShift::LTRS), (b'3', BaudotShift::FIGS)],
    [(b'\n', BaudotShift::LTRS), (b'\n', BaudotShift::FIGS)],
    [(b'A', BaudotShift::LTRS), (b'-', BaudotShift::FIGS)],
    [(b' ', BaudotShift::LTRS), (b' ', BaudotShift::FIGS)],
    [(b'S', BaudotShift::LTRS), (b'\'', BaudotShift::FIGS)],
    [(b'I', BaudotShift::LTRS), (b'8', BaudotShift::FIGS)],
    [(b'U', BaudotShift::LTRS), (b'7', BaudotShift::FIGS)],
    [(b'\r', BaudotShift::LTRS), (b'\r', BaudotShift::FIGS)],
    [(b'D', BaudotShift::LTRS), (b'\0', BaudotShift::FIGS)], // null represents ENQ
    [(b'R', BaudotShift::LTRS), (b'4', BaudotShift::FIGS)],
    [(b'J', BaudotShift::LTRS), (b'\0', BaudotShift::FIGS)], // null represents BEL
    [(b'N', BaudotShift::LTRS), (b',', BaudotShift::FIGS)],
    [(b'F', BaudotShift::LTRS), (b'!', BaudotShift::FIGS)],
    [(b'C', BaudotShift::LTRS), (b':', BaudotShift::FIGS)],
    [(b'K', BaudotShift::LTRS), (b'(', BaudotShift::FIGS)],
    [(b'T', BaudotShift::LTRS), (b'5', BaudotShift::FIGS)],
    [(b'Z', BaudotShift::LTRS), (b'+', BaudotShift::FIGS)],
    [(b'L', BaudotShift::LTRS), (b')', BaudotShift::FIGS)],
    [(b'W', BaudotShift::LTRS), (b'2', BaudotShift::FIGS)],
    [(b'H', BaudotShift::LTRS), (b'$', BaudotShift::FIGS)], // '$' should be '£', but '£' is not in ascii
    [(b'Y', BaudotShift::LTRS), (b'6', BaudotShift::FIGS)],
    [(b'P', BaudotShift::LTRS), (b'0', BaudotShift::FIGS)],
    [(b'Q', BaudotShift::LTRS), (b'1', BaudotShift::FIGS)],
    [(b'O', BaudotShift::LTRS), (b'9', BaudotShift::FIGS)],
    [(b'B', BaudotShift::LTRS), (b'?', BaudotShift::FIGS)],
    [(b'G', BaudotShift::LTRS), (b'&', BaudotShift::FIGS)],
    [(b'\0', BaudotShift::FIGS), (b'\0', BaudotShift::FIGS)], // null represents FIGS
    [(b'M', BaudotShift::LTRS), (b'.', BaudotShift::FIGS)],
    [(b'X', BaudotShift::LTRS), (b'/', BaudotShift::FIGS)],
    [(b'V', BaudotShift::LTRS), (b'=', BaudotShift::FIGS)],
    [(b'\0', BaudotShift::LTRS), (b'\0', BaudotShift::LTRS)], // null represents LTRS/DEL
];

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudotShift {
    LTRS = 0,
    FIGS = 1,
    Keep = 255,
}

#[derive(Debug, Clone, Copy)]
pub struct BaudotChar(BaudotShift, u8);

impl BaudotChar {
    pub fn to_ascii(self) -> (u8, BaudotShift) {
        BAUDOT_TO_ASCII[self.0 as usize][self.1 as usize]
    }
    pub fn from_ascii(ch: u8) -> Self {
        ASCII_TO_BAUDOT[ch as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineState {
    Empty,
    Writing,
    Reading,
}

pub struct BaudotStream<IN: PinId, OUT: PinId> {
    line_state: LineState,
    current_shift: BaudotShift,
    input: Pin<IN, FunctionSioInput, PullDown>,
    out: Pin<OUT, FunctionSioOutput, PullDown>,
    read_buf: u8,
    read_buf_len: u8,
    write_buf: [BaudotChar; WRITE_BUF_LENGTH],
    write_buf_char_pos: u8,
    write_buf_len: usize,
    start_bit_written: bool,
}

impl<IN: PinId, OUT: PinId> BaudotStream<IN, OUT> {
    pub fn new(
        input: Pin<IN, FunctionSioInput, PullDown>,
        out: Pin<OUT, FunctionSioOutput, PullDown>,
    ) -> Self {
        Self {
            line_state: Empty,
            current_shift: BaudotShift::LTRS,
            input,
            out,
            read_buf: 0,
            read_buf_len: 0,
            write_buf: [BaudotChar(BaudotShift::LTRS, 0); WRITE_BUF_LENGTH],
            write_buf_char_pos: 0,
            write_buf_len: 0,
            start_bit_written: false,
        }
    }

    pub fn poll_read(&mut self) -> (bool, Option<u8>, Option<MicrosDurationU32>) {
        let state = self.input.is_high().unwrap();
        if self.line_state == Empty {
            // read start bit
            if state {
                self.line_state = Reading;
                return (state, None, Some(BAUD_RATE));
            } else {
                return (state, None, None);
            }
        } else if self.line_state == Writing && self.start_bit_written {
            return (state, None, Some(BAUD_RATE));
        }

        self.read_buf |= (state as u8) << 4 - self.read_buf_len;
        self.read_buf_len += 1;
        if self.read_buf_len == 5 {
            let char = BAUDOT_TO_ASCII[self.read_buf as usize][self.current_shift as usize];
            if char.1 != BaudotShift::Keep {
                self.current_shift = char.1;
            }
            // multiply BAUD_RATE by 1.42 to account for stop bit
            return (state, Some(char.0), Some((BAUD_RATE * 142) / 100));
        }
        (state, None, Some(BAUD_RATE))
    }

    pub fn poll_write(&mut self) -> Option<MicrosDurationU32> {
        if self.write_buf_len == 0 {
            _ = self.out.set_high();
            return None;
        }
        if self.line_state == Reading {
            return None;
        } else if self.line_state == Empty {
            // write start bit
            self.line_state = Writing;
            _ = self.out.set_low();
            return Some(STOP_BIT_DURATION);
        }
        self.start_bit_written = true;
        let c = self.write_buf[0];
        let state = c.1 >> 4 - self.write_buf_char_pos & 1 == 1;
        _ = self.out.set_state(state.into());
        self.write_buf_char_pos += 1;
        if self.write_buf_char_pos == 5 {
            self.current_shift = c.0;
            self.write_buf.copy_within(1.., 0);
            self.write_buf_len -= 1;
        }
        Some(BAUD_RATE)
    }

    pub fn queue_write(&mut self, char: u8) {
        if self.write_buf_len == WRITE_BUF_LENGTH {
            return;
        }
        let char = BaudotChar::from_ascii(char);

        match (self.current_shift, char.0) {
            (BaudotShift::LTRS, BaudotShift::FIGS) => {
                if self.write_buf_len + 1 >= WRITE_BUF_LENGTH {
                    return;
                }
                self.write_buf[self.write_buf_len] = BaudotChar(BaudotShift::FIGS, 27);
                self.write_buf[self.write_buf_len + 1] = char;
            }
            (BaudotShift::FIGS, BaudotShift::LTRS) => {
                if self.write_buf_len + 1 >= WRITE_BUF_LENGTH {
                    return;
                }
                self.write_buf[self.write_buf_len] = BaudotChar(BaudotShift::LTRS, 31);
                self.write_buf[self.write_buf_len + 1] = char;
            }
            (_, _) => self.write_buf[self.write_buf_len] = char,
        }
    }
}
