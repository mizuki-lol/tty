use embedded_hal::digital::{InputPin, OutputPin};
use rp_pico::hal::{
    fugit::MicrosDurationU32,
    gpio::{FunctionSioInput, FunctionSioOutput, Pin, PinId, PullDown},
};

/// Length of the buffer holding characters meant for writing to the current loop.
const WRITE_BUF_LENGTH: usize = 1024;
/// The baud rate of the teletype. Defaults to 60 speed.
const BAUD_RATE: MicrosDurationU32 = MicrosDurationU32::millis(20);

/// Table for converting ASCII characters to their equivalents in ITA-2.
///
/// ```
/// let baudot = ASCII_TO_BAUDOT[b'a'];
///
/// // the character 'a' is in the Figure set.
/// assert_eq!(baudot.0, BaudotShift::Figs);
/// // this is the bit representation in ITA-2
/// assert_eq!(baudot.1, 0b000_11);
///
/// let ascii = BAUDOT_TO_ASCII[b.1][b.0];
/// assert_eq!(ascii.0, b'A');
/// ```
#[allow(clippy::unusual_byte_groupings)]
const ASCII_TO_BAUDOT: [BaudotChar; 256] = [
    BaudotChar(BaudotShift::Keep, 0),        // null
    BaudotChar(BaudotShift::Keep, 0),        // start of heading
    BaudotChar(BaudotShift::Keep, 0),        // start of text
    BaudotChar(BaudotShift::Keep, 0),        // end of text
    BaudotChar(BaudotShift::Keep, 0),        // end of transmision
    BaudotChar(BaudotShift::Figs, 0b10_010), // enquiry
    BaudotChar(BaudotShift::Keep, 0),        // acknowledge
    BaudotChar(BaudotShift::Figs, 0b010_11), // bell
    BaudotChar(BaudotShift::Keep, 0),        // backspace
    BaudotChar(BaudotShift::Keep, 0),        // tab
    BaudotChar(BaudotShift::Keep, 0b000_10), // new line
    BaudotChar(BaudotShift::Keep, 0),        // vertical tab
    BaudotChar(BaudotShift::Keep, 0),        // form feed
    BaudotChar(BaudotShift::Keep, 0b010_00), // carriage return
    BaudotChar(BaudotShift::Keep, 0),        // shift out
    BaudotChar(BaudotShift::Keep, 0),        // shift in
    BaudotChar(BaudotShift::Keep, 0),        // data link escape
    BaudotChar(BaudotShift::Keep, 0),        // device control 1
    BaudotChar(BaudotShift::Keep, 0),        // device control 2
    BaudotChar(BaudotShift::Keep, 0),        // device control 3
    BaudotChar(BaudotShift::Keep, 0),        // device control 4
    BaudotChar(BaudotShift::Keep, 0),        // negative acknowledge
    BaudotChar(BaudotShift::Keep, 0),        // synchronous idle
    BaudotChar(BaudotShift::Keep, 0),        // end of transmission
    BaudotChar(BaudotShift::Keep, 0),        // cancel
    BaudotChar(BaudotShift::Keep, 0),        // end of medium
    BaudotChar(BaudotShift::Keep, 0),        // substitute
    BaudotChar(BaudotShift::Keep, 0),        // escape
    BaudotChar(BaudotShift::Keep, 0),        // file separator
    BaudotChar(BaudotShift::Keep, 0),        // group separator
    BaudotChar(BaudotShift::Keep, 0),        // record separator
    BaudotChar(BaudotShift::Keep, 0),        // unit separator
    BaudotChar(BaudotShift::Keep, 0b001_00), // space
    BaudotChar(BaudotShift::Keep, 0),        // !
    BaudotChar(BaudotShift::Figs, 0b100_01), // "
    BaudotChar(BaudotShift::Figs, 0b101_00), // #
    BaudotChar(BaudotShift::Figs, 0b010_01), // $
    BaudotChar(BaudotShift::Keep, 0),        // %
    BaudotChar(BaudotShift::Figs, 0b110_10), // &
    BaudotChar(BaudotShift::Figs, 0b010_11), // '
    BaudotChar(BaudotShift::Figs, 0b011_11), // (
    BaudotChar(BaudotShift::Figs, 0b100_10), // )
    BaudotChar(BaudotShift::Keep, 0),        // *
    BaudotChar(BaudotShift::Figs, 0b100_01), // +
    BaudotChar(BaudotShift::Figs, 0b011_00), // ,
    BaudotChar(BaudotShift::Figs, 0b000_11), // -
    BaudotChar(BaudotShift::Figs, 0b111_00), // .
    BaudotChar(BaudotShift::Figs, 0b111_01), // /
    BaudotChar(BaudotShift::Figs, 0b101_10), // 0
    BaudotChar(BaudotShift::Figs, 0b101_11), // 1
    BaudotChar(BaudotShift::Figs, 0b100_11), // 2
    BaudotChar(BaudotShift::Figs, 0b000_01), // 3
    BaudotChar(BaudotShift::Figs, 0b010_10), // 4
    BaudotChar(BaudotShift::Figs, 0b100_00), // 5
    BaudotChar(BaudotShift::Figs, 0b101_01), // 6
    BaudotChar(BaudotShift::Figs, 0b001_11), // 7
    BaudotChar(BaudotShift::Figs, 0b001_10), // 8
    BaudotChar(BaudotShift::Figs, 0b110_00), // 9
    BaudotChar(BaudotShift::Figs, 0b011_10), // :
    BaudotChar(BaudotShift::Figs, 0b111_10), // ;
    BaudotChar(BaudotShift::Keep, 0),        // <
    BaudotChar(BaudotShift::Figs, 0b111_10), // =
    BaudotChar(BaudotShift::Keep, 0),        // >
    BaudotChar(BaudotShift::Figs, 0b110_01), // ?
    BaudotChar(BaudotShift::Keep, 0),        // @
    BaudotChar(BaudotShift::Ltrs, 0b000_11), // A
    BaudotChar(BaudotShift::Ltrs, 0b110_01), // B
    BaudotChar(BaudotShift::Ltrs, 0b011_10), // C
    BaudotChar(BaudotShift::Ltrs, 0b010_01), // D
    BaudotChar(BaudotShift::Ltrs, 0b000_01), // E
    BaudotChar(BaudotShift::Ltrs, 0b011_01), // F
    BaudotChar(BaudotShift::Ltrs, 0b110_10), // G
    BaudotChar(BaudotShift::Ltrs, 0b101_00), // H
    BaudotChar(BaudotShift::Ltrs, 0b001_10), // I
    BaudotChar(BaudotShift::Ltrs, 0b010_11), // J
    BaudotChar(BaudotShift::Ltrs, 0b011_11), // K
    BaudotChar(BaudotShift::Ltrs, 0b100_10), // L
    BaudotChar(BaudotShift::Ltrs, 0b111_00), // M
    BaudotChar(BaudotShift::Ltrs, 0b011_00), // N
    BaudotChar(BaudotShift::Ltrs, 0b110_00), // O
    BaudotChar(BaudotShift::Ltrs, 0b101_10), // P
    BaudotChar(BaudotShift::Ltrs, 0b101_11), // Q
    BaudotChar(BaudotShift::Ltrs, 0b010_10), // R
    BaudotChar(BaudotShift::Ltrs, 0b001_01), // S
    BaudotChar(BaudotShift::Ltrs, 0b100_00), // T
    BaudotChar(BaudotShift::Ltrs, 0b001_11), // U
    BaudotChar(BaudotShift::Ltrs, 0b111_10), // V
    BaudotChar(BaudotShift::Ltrs, 0b100_11), // W
    BaudotChar(BaudotShift::Ltrs, 0b111_01), // X
    BaudotChar(BaudotShift::Ltrs, 0b101_01), // Y
    BaudotChar(BaudotShift::Ltrs, 0b100_01), // Z
    BaudotChar(BaudotShift::Keep, 0),        // [
    BaudotChar(BaudotShift::Keep, 0),        // \
    BaudotChar(BaudotShift::Keep, 0),        // ]
    BaudotChar(BaudotShift::Keep, 0),        // ^
    BaudotChar(BaudotShift::Figs, 0b000_11), // _
    BaudotChar(BaudotShift::Keep, 0),        // `
    BaudotChar(BaudotShift::Ltrs, 0b000_11), // a
    BaudotChar(BaudotShift::Ltrs, 0b110_01), // b
    BaudotChar(BaudotShift::Ltrs, 0b011_10), // c
    BaudotChar(BaudotShift::Ltrs, 0b010_01), // d
    BaudotChar(BaudotShift::Ltrs, 0b000_01), // e
    BaudotChar(BaudotShift::Ltrs, 0b011_01), // f
    BaudotChar(BaudotShift::Ltrs, 0b110_10), // g
    BaudotChar(BaudotShift::Ltrs, 0b101_00), // h
    BaudotChar(BaudotShift::Ltrs, 0b001_10), // i
    BaudotChar(BaudotShift::Ltrs, 0b010_11), // j
    BaudotChar(BaudotShift::Ltrs, 0b011_11), // k
    BaudotChar(BaudotShift::Ltrs, 0b100_10), // l
    BaudotChar(BaudotShift::Ltrs, 0b111_00), // m
    BaudotChar(BaudotShift::Ltrs, 0b011_00), // n
    BaudotChar(BaudotShift::Ltrs, 0b110_00), // o
    BaudotChar(BaudotShift::Ltrs, 0b101_10), // p
    BaudotChar(BaudotShift::Ltrs, 0b101_11), // q
    BaudotChar(BaudotShift::Ltrs, 0b010_10), // r
    BaudotChar(BaudotShift::Ltrs, 0b001_01), // s
    BaudotChar(BaudotShift::Ltrs, 0b100_00), // t
    BaudotChar(BaudotShift::Ltrs, 0b001_11), // u
    BaudotChar(BaudotShift::Ltrs, 0b111_10), // v
    BaudotChar(BaudotShift::Ltrs, 0b100_11), // w
    BaudotChar(BaudotShift::Ltrs, 0b111_01), // x
    BaudotChar(BaudotShift::Ltrs, 0b101_01), // y
    BaudotChar(BaudotShift::Ltrs, 0b100_01), // z
    BaudotChar(BaudotShift::Keep, 0),        // {
    BaudotChar(BaudotShift::Keep, 0),        // |
    BaudotChar(BaudotShift::Keep, 0),        // }
    BaudotChar(BaudotShift::Keep, 0),        // ~
    BaudotChar(BaudotShift::Ltrs, 0b11_111), // delete
    BaudotChar(BaudotShift::Keep, 0),        // --- rest is unused ---
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

/// Table for converting characters from ITA-2 to ascii as defined by the table here
/// https://en.wikipedia.org/wiki/Baudot_code#ITA_2_and_US-TTY.
/// The first dimension of this array is meant to be indexed with the baudot char and
/// the second with the current shift state of the teleprinter.
/// The first value of the touple is the converted character whereas the second is
/// the next shift state of the teleprinter after printing this char.
const BAUDOT_TO_ASCII: [[(u8, BaudotShift); 2]; 32] = [
    [(b'\0', BaudotShift::Ltrs), (b'\0', BaudotShift::Figs)],
    [(b'E', BaudotShift::Ltrs), (b'3', BaudotShift::Figs)],
    [(b'\n', BaudotShift::Ltrs), (b'\n', BaudotShift::Figs)],
    [(b'A', BaudotShift::Ltrs), (b'-', BaudotShift::Figs)],
    [(b' ', BaudotShift::Ltrs), (b' ', BaudotShift::Figs)],
    [(b'S', BaudotShift::Ltrs), (b'\'', BaudotShift::Figs)],
    [(b'I', BaudotShift::Ltrs), (b'8', BaudotShift::Figs)],
    [(b'U', BaudotShift::Ltrs), (b'7', BaudotShift::Figs)],
    [(b'\r', BaudotShift::Ltrs), (b'\r', BaudotShift::Figs)],
    [(b'D', BaudotShift::Ltrs), (b'\0', BaudotShift::Figs)], // null represents ENQ
    [(b'R', BaudotShift::Ltrs), (b'4', BaudotShift::Figs)],
    [(b'J', BaudotShift::Ltrs), (b'\0', BaudotShift::Figs)], // null represents BEL
    [(b'N', BaudotShift::Ltrs), (b',', BaudotShift::Figs)],
    [(b'F', BaudotShift::Ltrs), (b'!', BaudotShift::Figs)],
    [(b'C', BaudotShift::Ltrs), (b':', BaudotShift::Figs)],
    [(b'K', BaudotShift::Ltrs), (b'(', BaudotShift::Figs)],
    [(b'T', BaudotShift::Ltrs), (b'5', BaudotShift::Figs)],
    [(b'Z', BaudotShift::Ltrs), (b'+', BaudotShift::Figs)],
    [(b'L', BaudotShift::Ltrs), (b')', BaudotShift::Figs)],
    [(b'W', BaudotShift::Ltrs), (b'2', BaudotShift::Figs)],
    [(b'H', BaudotShift::Ltrs), (b'$', BaudotShift::Figs)], // '$' should be '£', but '£' is not in ascii
    [(b'Y', BaudotShift::Ltrs), (b'6', BaudotShift::Figs)],
    [(b'P', BaudotShift::Ltrs), (b'0', BaudotShift::Figs)],
    [(b'Q', BaudotShift::Ltrs), (b'1', BaudotShift::Figs)],
    [(b'O', BaudotShift::Ltrs), (b'9', BaudotShift::Figs)],
    [(b'B', BaudotShift::Ltrs), (b'?', BaudotShift::Figs)],
    [(b'G', BaudotShift::Ltrs), (b'&', BaudotShift::Figs)],
    [(b'\0', BaudotShift::Figs), (b'\0', BaudotShift::Figs)], // null represents FIGS
    [(b'M', BaudotShift::Ltrs), (b'.', BaudotShift::Figs)],
    [(b'X', BaudotShift::Ltrs), (b'/', BaudotShift::Figs)],
    [(b'V', BaudotShift::Ltrs), (b'=', BaudotShift::Figs)],
    [(b'\0', BaudotShift::Ltrs), (b'\0', BaudotShift::Ltrs)], // null represents LTRS/DEL
];

/// The shift state of the teleprinter
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudotShift {
    /// LTRS
    Ltrs = 0,
    /// FIGS
    Figs = 1,
    /// This value says to keep the current state when writing this char to the current loop.
    Keep = 255,
}

/// An index to the [BAUDOT_TO_ASCII] table.
#[derive(Debug, Clone, Copy)]
pub struct BaudotChar(pub BaudotShift, pub u8);

impl BaudotChar {
    pub fn to_ascii(self) -> (u8, BaudotShift) {
        if self.0 == BaudotShift::Keep {
            return (0, BaudotShift::Keep);
        }
        BAUDOT_TO_ASCII[self.1 as usize][self.0 as usize]
    }
    pub fn from_ascii(ch: u8) -> Self {
        ASCII_TO_BAUDOT[ch as usize]
    }
}

/// An abstraction over the current loop used for communicating between teletypes.
///
/// To write to characters first queue them with [BaudotStream::queue_write] and then
/// poll the writes with [BaudotStream::poll_write] with delays in between.
///
/// Reading is done using the [BaudotStream::poll_read] method.
pub struct BaudotStream<IN: PinId, OUT: PinId> {
    current_shift: BaudotShift,
    input: Pin<IN, FunctionSioInput, PullDown>,
    out: Pin<OUT, FunctionSioOutput, PullDown>,

    reading: bool,
    read_buf: u8,
    read_buf_len: u8,

    writing: bool,
    write_buf: [u8; WRITE_BUF_LENGTH],
    write_buf_char_pos: u8,
    write_buf_len: usize,
    write_buf_end_shift: BaudotShift,
    start_bit_written: bool,
}

impl<IN: PinId, OUT: PinId> BaudotStream<IN, OUT> {
    /// Creates an abstraction over the logic of switching the current loop.
    /// [input] is a pin that is used for reading the state of the current loop.
    /// A logical 1 correspons to the marking state and 0 to the spacing state.
    /// [output] is a pin used for for controlling the opening and closing of the
    /// current loop. A logical 1 means that the loop is closed (current is flowing,
    /// this is the marking state) and 0 means that the loop should be open (no current
    /// is flowing, the state is marking).
    pub fn new(
        input: Pin<IN, FunctionSioInput, PullDown>,
        out: Pin<OUT, FunctionSioOutput, PullDown>,
    ) -> Self {
        Self {
            current_shift: BaudotShift::Ltrs,
            input,
            out,

            reading: false,
            read_buf: 0,
            read_buf_len: 0,

            writing: false,
            write_buf: [0; WRITE_BUF_LENGTH],
            write_buf_char_pos: 0,
            write_buf_len: 0,
            write_buf_end_shift: BaudotShift::Ltrs,
            start_bit_written: false,
        }
    }

    /// Try reading some data from the current loop.
    ///
    /// Returns the state of the current loop, an ascii character if one was
    /// done reading and a delay when should the next poll happen (if `None` is return
    /// an another poll may happen as soon as possible).
    pub fn poll_read(&mut self) -> (bool, Option<u8>, Option<MicrosDurationU32>) {
        let state = self.input.is_high().unwrap();

        if !self.reading {
            // read start bit
            if !state {
                self.reading = true;
                return (state, None, Some(BAUD_RATE));
            } else {
                return (state, None, None);
            }
        }

        if self.read_buf_len == 5 {
            let char = BaudotChar(self.current_shift, self.read_buf).to_ascii();
            let mut ascii = Some(char.0);
            // don't send anything to uart when changing shift state
            if char.1 != BaudotShift::Keep && self.current_shift != char.1 {
                self.current_shift = char.1;
                ascii = None;
            }
            self.reading = false;
            self.read_buf = 0;
            self.read_buf_len = 0;
            // wait for last stop bit
            return (state, ascii, Some(BAUD_RATE));
        }

        let bit = if state { 1 } else { 0 };
        self.read_buf |= bit << self.read_buf_len;
        self.read_buf_len += 1;
        (state, None, Some(BAUD_RATE))
    }

    /// Try writing some data to the current loop.
    /// If a duration is returned you must call the function again after at least that duration.
    pub fn poll_write(&mut self) -> Option<MicrosDurationU32> {
        if self.write_buf_len == 0 {
            _ = self.out.set_high();
            return None;
        }

        if !self.writing {
            // write start bit
            self.writing = true;
            self.start_bit_written = true;
            _ = self.out.set_low();
            return Some(BAUD_RATE);
        }

        if self.write_buf_char_pos == 5 {
            self.writing = false;
            self.write_buf.copy_within(1..self.write_buf_len, 0);
            self.write_buf_len -= 1;
            self.write_buf_char_pos = 0;
            self.start_bit_written = false;

            // write stop bit(s)
            _ = self.out.set_high();
            return Some((BAUD_RATE * 142) / 100);
        }

        let c = self.write_buf[0];
        let state = c >> self.write_buf_char_pos & 1 == 1;
        _ = self.out.set_state(state.into());
        self.write_buf_char_pos += 1;
        Some(BAUD_RATE)
    }

    /// Queue some data to be written to the current loop.
    pub fn queue_write(&mut self, char: u8) {
        if self.write_buf_len == WRITE_BUF_LENGTH {
            return;
        }
        let char = BaudotChar::from_ascii(char);

        match (self.write_buf_end_shift, char.0) {
            (BaudotShift::Ltrs, BaudotShift::Figs) => {
                if self.write_buf_len + 1 == WRITE_BUF_LENGTH {
                    return;
                }
                self.write_buf[self.write_buf_len] = 27;
                self.write_buf[self.write_buf_len + 1] = char.1;
                self.write_buf_len += 2;
                self.write_buf_end_shift = char.0;
            }
            (BaudotShift::Figs, BaudotShift::Ltrs) => {
                if self.write_buf_len + 1 == WRITE_BUF_LENGTH {
                    return;
                }
                self.write_buf[self.write_buf_len] = 31;
                self.write_buf[self.write_buf_len + 1] = char.1;
                self.write_buf_len += 2;
                self.write_buf_end_shift = char.0;
            }
            (_, _) => {
                self.write_buf[self.write_buf_len] = char.1;
                self.write_buf_len += 1;
            }
        }
    }
}
