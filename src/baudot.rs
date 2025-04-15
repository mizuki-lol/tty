use embedded_hal::digital::{InputPin, OutputPin};
use rp2040_hal::gpio::{FunctionSioInput, FunctionSioOutput, Pin, PinId, PullDown};

const WRITE_BUF_LENGTH: usize = 1024;

const ASCII_TO_BAUDOT: [Option<BaudotChar>; 256] = [
    Some(BaudotChar(BaudotShift::LTRS, 0)), // null
    Some(BaudotChar(BaudotShift::LTRS, 0)), // start of heading
    Some(BaudotChar(BaudotShift::LTRS, 0)), // start of text
    Some(BaudotChar(BaudotShift::LTRS, 0)), // end of text
    Some(BaudotChar(BaudotShift::LTRS, 0)), // end of transmision
    Some(BaudotChar(BaudotShift::LTRS, 0)), // enquiry
    Some(BaudotChar(BaudotShift::LTRS, 0)), // acknowledge
    Some(BaudotChar(BaudotShift::LTRS, 0)), // bell
    Some(BaudotChar(BaudotShift::LTRS, 0)), // backspace
    Some(BaudotChar(BaudotShift::LTRS, 0)), // tab
    Some(BaudotChar(BaudotShift::LTRS, 0)), // new line
    Some(BaudotChar(BaudotShift::LTRS, 0)), // vertical tab
    Some(BaudotChar(BaudotShift::LTRS, 0)), // form feed
    Some(BaudotChar(BaudotShift::LTRS, 0)), // carriage return
    Some(BaudotChar(BaudotShift::LTRS, 0)), // shift out
    Some(BaudotChar(BaudotShift::LTRS, 0)), // shift in
    Some(BaudotChar(BaudotShift::LTRS, 0)), // data link escape
    Some(BaudotChar(BaudotShift::LTRS, 0)), // device control 1
    Some(BaudotChar(BaudotShift::LTRS, 0)), // device control 2
    Some(BaudotChar(BaudotShift::LTRS, 0)), // device control 3
    Some(BaudotChar(BaudotShift::LTRS, 0)), // device control 4
    Some(BaudotChar(BaudotShift::LTRS, 0)), // negative acknowledge
    Some(BaudotChar(BaudotShift::LTRS, 0)), // synchronous idle
    Some(BaudotChar(BaudotShift::LTRS, 0)), // end of transmission
    Some(BaudotChar(BaudotShift::LTRS, 0)), // cancel
    Some(BaudotChar(BaudotShift::LTRS, 0)), // end of medium
    Some(BaudotChar(BaudotShift::LTRS, 0)), // substitute
    Some(BaudotChar(BaudotShift::LTRS, 0)), // escape
    Some(BaudotChar(BaudotShift::LTRS, 0)), // file separator
    Some(BaudotChar(BaudotShift::LTRS, 0)), // group separator
    Some(BaudotChar(BaudotShift::LTRS, 0)), // record separator
    Some(BaudotChar(BaudotShift::LTRS, 0)), // unit separator
    Some(BaudotChar(BaudotShift::LTRS, 0)), // space
    Some(BaudotChar(BaudotShift::LTRS, 0)), // !
    Some(BaudotChar(BaudotShift::LTRS, 0)), // "
    Some(BaudotChar(BaudotShift::LTRS, 0)), // #
    Some(BaudotChar(BaudotShift::LTRS, 0)), // $
    Some(BaudotChar(BaudotShift::LTRS, 0)), // %
    Some(BaudotChar(BaudotShift::LTRS, 0)), // &
    Some(BaudotChar(BaudotShift::LTRS, 0)), // '
    Some(BaudotChar(BaudotShift::LTRS, 0)), // (
    Some(BaudotChar(BaudotShift::LTRS, 0)), // )
    Some(BaudotChar(BaudotShift::LTRS, 0)), // *
    Some(BaudotChar(BaudotShift::LTRS, 0)), // +
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ,
    Some(BaudotChar(BaudotShift::LTRS, 0)), // -
    Some(BaudotChar(BaudotShift::LTRS, 0)), // .
    Some(BaudotChar(BaudotShift::LTRS, 0)), // /
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 0
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 1
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 2
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 3
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 4
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 5
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 6
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 7
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 8
    Some(BaudotChar(BaudotShift::LTRS, 0)), // 9
    Some(BaudotChar(BaudotShift::LTRS, 0)), // :
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ;
    Some(BaudotChar(BaudotShift::LTRS, 0)), // <
    Some(BaudotChar(BaudotShift::LTRS, 0)), // =
    Some(BaudotChar(BaudotShift::LTRS, 0)), // >
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ?
    Some(BaudotChar(BaudotShift::LTRS, 0)), // @
    Some(BaudotChar(BaudotShift::LTRS, 0)), // A
    Some(BaudotChar(BaudotShift::LTRS, 0)), // B
    Some(BaudotChar(BaudotShift::LTRS, 0)), // C
    Some(BaudotChar(BaudotShift::LTRS, 0)), // D
    Some(BaudotChar(BaudotShift::LTRS, 0)), // E
    Some(BaudotChar(BaudotShift::LTRS, 0)), // F
    Some(BaudotChar(BaudotShift::LTRS, 0)), // G
    Some(BaudotChar(BaudotShift::LTRS, 0)), // H
    Some(BaudotChar(BaudotShift::LTRS, 0)), // I
    Some(BaudotChar(BaudotShift::LTRS, 0)), // J
    Some(BaudotChar(BaudotShift::LTRS, 0)), // K
    Some(BaudotChar(BaudotShift::LTRS, 0)), // L
    Some(BaudotChar(BaudotShift::LTRS, 0)), // M
    Some(BaudotChar(BaudotShift::LTRS, 0)), // N
    Some(BaudotChar(BaudotShift::LTRS, 0)), // O
    Some(BaudotChar(BaudotShift::LTRS, 0)), // P
    Some(BaudotChar(BaudotShift::LTRS, 0)), // Q
    Some(BaudotChar(BaudotShift::LTRS, 0)), // R
    Some(BaudotChar(BaudotShift::LTRS, 0)), // S
    Some(BaudotChar(BaudotShift::LTRS, 0)), // T
    Some(BaudotChar(BaudotShift::LTRS, 0)), // U
    Some(BaudotChar(BaudotShift::LTRS, 0)), // V
    Some(BaudotChar(BaudotShift::LTRS, 0)), // W
    Some(BaudotChar(BaudotShift::LTRS, 0)), // X
    Some(BaudotChar(BaudotShift::LTRS, 0)), // Y
    Some(BaudotChar(BaudotShift::LTRS, 0)), // Z
    Some(BaudotChar(BaudotShift::LTRS, 0)), // [
    Some(BaudotChar(BaudotShift::LTRS, 0)), // \
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ]
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ^
    Some(BaudotChar(BaudotShift::LTRS, 0)), // _
    Some(BaudotChar(BaudotShift::LTRS, 0)), // `
    Some(BaudotChar(BaudotShift::LTRS, 0)), // a
    Some(BaudotChar(BaudotShift::LTRS, 0)), // b
    Some(BaudotChar(BaudotShift::LTRS, 0)), // c
    Some(BaudotChar(BaudotShift::LTRS, 0)), // d
    Some(BaudotChar(BaudotShift::LTRS, 0)), // e
    Some(BaudotChar(BaudotShift::LTRS, 0)), // f
    Some(BaudotChar(BaudotShift::LTRS, 0)), // g
    Some(BaudotChar(BaudotShift::LTRS, 0)), // h
    Some(BaudotChar(BaudotShift::LTRS, 0)), // i
    Some(BaudotChar(BaudotShift::LTRS, 0)), // j
    Some(BaudotChar(BaudotShift::LTRS, 0)), // k
    Some(BaudotChar(BaudotShift::LTRS, 0)), // l
    Some(BaudotChar(BaudotShift::LTRS, 0)), // m
    Some(BaudotChar(BaudotShift::LTRS, 0)), // n
    Some(BaudotChar(BaudotShift::LTRS, 0)), // o
    Some(BaudotChar(BaudotShift::LTRS, 0)), // p
    Some(BaudotChar(BaudotShift::LTRS, 0)), // q
    Some(BaudotChar(BaudotShift::LTRS, 0)), // r
    Some(BaudotChar(BaudotShift::LTRS, 0)), // s
    Some(BaudotChar(BaudotShift::LTRS, 0)), // t
    Some(BaudotChar(BaudotShift::LTRS, 0)), // u
    Some(BaudotChar(BaudotShift::LTRS, 0)), // v
    Some(BaudotChar(BaudotShift::LTRS, 0)), // w
    Some(BaudotChar(BaudotShift::LTRS, 0)), // x
    Some(BaudotChar(BaudotShift::LTRS, 0)), // y
    Some(BaudotChar(BaudotShift::LTRS, 0)), // z
    Some(BaudotChar(BaudotShift::LTRS, 0)), // {
    Some(BaudotChar(BaudotShift::LTRS, 0)), // |
    Some(BaudotChar(BaudotShift::LTRS, 0)), // }
    Some(BaudotChar(BaudotShift::LTRS, 0)), // ~
    Some(BaudotChar(BaudotShift::LTRS, 0)), // delete
    None,                                   // --- rest is unused ---
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
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
#[derive(Debug, Clone, Copy)]
pub enum BaudotShift {
    LTRS = 0,
    FIGS = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct BaudotChar(BaudotShift, u8);

impl BaudotChar {
    pub fn to_ascii(self) -> (u8, BaudotShift) {
        BAUDOT_TO_ASCII[self.0 as usize][self.1 as usize]
    }
    pub fn from_ascii(ch: u8) -> Option<Self> {
        ASCII_TO_BAUDOT[ch as usize]
    }
}

pub struct BaudotStream<IN: PinId, OUT: PinId> {
    current_shift: BaudotShift,
    input: Pin<IN, FunctionSioInput, PullDown>,
    out: Pin<OUT, FunctionSioOutput, PullDown>,
    read_buf: u8,
    read_buf_len: u8,
    write_buf: [BaudotChar; WRITE_BUF_LENGTH],
    write_buf_char_pos: u8,
    write_buf_len: usize,
}

impl<IN: PinId, OUT: PinId> BaudotStream<IN, OUT> {
    pub fn new(
        input: Pin<IN, FunctionSioInput, PullDown>,
        out: Pin<OUT, FunctionSioOutput, PullDown>,
    ) -> Self {
        Self {
            current_shift: BaudotShift::LTRS,
            input,
            out,
            read_buf: 0,
            read_buf_len: 0,
            write_buf: [BaudotChar(BaudotShift::LTRS, 0); WRITE_BUF_LENGTH],
            write_buf_char_pos: 0,
            write_buf_len: 0,
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        let state = self.input.is_high().unwrap();
        self.read_buf |= (state as u8) << 4 - self.read_buf_len;
        self.read_buf_len += 1;
        if self.read_buf_len == 5 {
            let char = BAUDOT_TO_ASCII[self.read_buf as usize][self.current_shift as usize];
            self.current_shift = char.1;
            return Some(char.0);
        }
        None
    }

    pub fn write(&mut self) {
        if self.write_buf_len == 0 {
            return;
        }
        let c = self.write_buf[0];
        let state = c.1 >> 4 - self.write_buf_char_pos & 1 == 1;
        self.out.set_state(state.into()).unwrap();
        self.write_buf_char_pos += 1;
        if self.write_buf_char_pos == 5 {
            self.current_shift = c.0;
            self.write_buf.copy_within(1.., 0);
            self.write_buf_len -= 1;
        }
    }

    pub fn append_to_buf(&mut self, char: u8) {
        if self.write_buf_len == WRITE_BUF_LENGTH {
            return;
        }
        let Some(char) = BaudotChar::from_ascii(char) else {
            return;
        };
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
