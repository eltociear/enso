//! The data hold by the text buffer. Under the hood it is implemented as an efficient string rope.

use crate::prelude::*;
use crate::unit::*;

use crate::prelude::fmt::Formatter;
use crate::range::Range;
use crate::range::RangeBounds;
use crate::rope;
use crate::rope::Rope;

use enso_types::min;



// ============
// === Text ===
// ============

/// Efficient, immutable text container used by the text buffer. Implemented as a rope under the
/// hood. Use `TextCell` if you are looking for an internally mutable version.
///
/// A [rope](https://en.wikipedia.org/wiki/Rope_(data_structure)) is a data structure for strings,
/// specialized for incremental editing operations. Most operations (such as insert, delete,
/// substring) are O(log n). This module provides an immutable (also known as
/// [persistent](https://en.wikipedia.org/wiki/Persistent_data_structure)) version of Ropes, and if
/// there are many copies of similar strings, the common parts are shared.
///
/// Internally, the implementation uses thread safe reference counting. Mutations are generally
/// copy-on-write, though in-place edits are supported as an optimization when only one reference
/// exists, making the implementation as efficient as a mutable version.
///
/// This type provides multiple `From` implementations for easy conversions from string-like types,
/// and vice-versa.
///
/// Please note that the underlying rope implementation comes from `xi-rope` crate which does not
/// use strong types for all units (like line number, column number, byte offset), so part of
/// responsibility of this struct is to wrap the underlying API with strong types introduced in this
/// library.
#[derive(Debug, Clone, Default, Deref)]
#[allow(missing_docs)]
pub struct Text {
    pub rope: Rope,
}
impl_clone_ref_as_clone!(Text);



// =========================
// === Generic Utilities ===
// =========================

// === Constructors and Info ===

impl Text {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Check whether the text is empty.
    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    /// Return text narrowed to the given range.
    pub fn sub(&self, range: impl RangeBounds) -> Text {
        let range = self.crop_byte_range(range);
        let rope = self.rope.subseq(range.into_rope_interval());
        Text { rope }
    }

    /// The number of grapheme clusters in this text.
    pub fn grapheme_count(&self) -> usize {
        let mut offset = 0;
        let mut count = 0;
        while let Some(off) = self.rope.next_grapheme_offset(offset) {
            offset = off;
            count += 1;
        }
        count
    }

    /// Return the len of the text in bytes.
    pub fn byte_size(&self) -> UBytes {
        UBytes(self.rope.len())
    }

    /// Range of the text in bytes.
    pub fn byte_range(&self) -> Range<UBytes> {
        (..self.byte_size()).into()
    }

    /// Constraint the provided byte range, so it will be contained of the range of this data. This
    /// ensures that the resulting byte range will be valid for operations on this data.
    pub fn crop_byte_range(&self, range: impl RangeBounds) -> Range<UBytes> {
        range.with_upper_bound(self.byte_size())
    }

    /// Constraint the provided location, so it will be contained of the range of this data. This
    /// ensures that the resulting location will be valid for operations on this data.
    pub fn snap_location(&self, location: Location<UBytes>) -> Location<UBytes> {
        use BoundsError::*;
        match self.validate_line_index(location.line) {
            Err(TooSmall) => self.first_line_start_location(),
            Err(TooBig) => self.last_line_end_location(),
            Ok(line) => {
                let byte_offset =
                    min(location.offset, self.end_byte_offset_of_line_index(line).unwrap());
                Location(line, byte_offset)
            }
        }
    }

    /// Return the offset to the next codepoint if any. See the [`crate`] documentation to learn
    /// more about codepoints.
    pub fn next_codepoint_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.rope.next_codepoint_offset(offset.value).map(UBytes)
    }

    /// Return the offset to the previous codepoint if any. See the [`crate`] documentation to learn
    /// more about codepoints.
    pub fn prev_codepoint_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.rope.prev_codepoint_offset(offset.value).map(UBytes)
    }

    /// Return the offset to the next grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn next_grapheme_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.rope.next_grapheme_offset(offset.value).map(UBytes)
    }

    /// Return the offset to the previous grapheme if any. See the documentation of the library to
    /// learn more about graphemes.
    pub fn prev_grapheme_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.rope.prev_grapheme_offset(offset.value).map(UBytes)
    }

    /// An iterator over the lines of a rope.
    ///
    /// Lines are ended with either Unix (`\n`) or MS-DOS (`\r\n`) style line endings. The line
    /// ending is stripped from the resulting string. The final line ending is optional.
    pub fn lines<T: rope::IntervalBounds>(&self, range: T) -> rope::Lines {
        self.rope.lines(range)
    }

    /// Replaces the provided range with the provided text.
    pub fn replace(&mut self, range: impl RangeBounds, text: impl Into<Text>) {
        let text = text.into();
        let range = self.crop_byte_range(range);
        self.rope.edit(range.into_rope_interval(), text.rope);
    }

    /// Apply the given change on the current text.
    ///
    /// See also [`Self::replace`].
    pub fn apply_change(&mut self, change: Change<UBytes, impl Into<Text>>) {
        self.replace(change.range, change.text)
    }
}


// === First Line ===

impl Text {
    /// The first valid line index in this text.
    pub fn first_line_index(&self) -> Line {
        Line(0)
    }

    /// The first valid line byte offset in this text.
    pub fn first_line_byte_offset(&self) -> UBytes {
        0.ubytes()
    }

    /// The start column of the first line.
    pub fn first_line_start_column(&self) -> CodePointIndex {
        0.code_point_index()
    }

    /// The start location of the first line.
    pub fn first_line_start_location(&self) -> Location<UBytes> {
        let line = self.first_line_index();
        let byte_offset = self.first_line_byte_offset();
        Location(line, byte_offset)
    }
}


// === Last Line ===

impl Text {
    /// The last valid line index in this text. If the text ends with the newline character,
    /// it means that there is an empty last line.
    pub fn last_line_index(&self) -> Line {
        (self.rope.measure::<rope::metric::Lines>()).into()
    }

    /// The last valid line byte offset in this text. If the text ends with the newline character,
    /// it means that there is an empty last line.
    pub fn last_line_byte_offset(&self) -> UBytes {
        self.byte_offset_of_line_index_unchecked(self.last_line_index())
    }

    /// The start column of the last line.
    pub fn last_line_start_column(&self) -> CodePointIndex {
        0.code_point_index()
    }

    /// The start location of the last line.
    pub fn last_line_start_location(&self) -> Location<UBytes> {
        let line = self.last_line_index();
        let byte_offset = UBytes(0);
        Location(line, byte_offset)
    }

    // /// The last column number of the last line.
    // pub fn last_line_end_column(&self) -> CodePointIndex {
    //     self.column_of_byte_offset(self.byte_size()).unwrap()
    // }

    /// The end location of the last line.
    pub fn last_line_end_column_byte_offset(&self) -> UBytes {
        self.line_byte_offset_of_byte_offset(self.byte_size()).unwrap()
    }

    /// The byte offset of the end of the last line. Equal to the byte size of the whole text.
    pub fn last_line_end_byte_offset(&self) -> UBytes {
        self.byte_size()
    }

    /// The location of the last character in the text.
    pub fn last_line_end_location(&self) -> Location<UBytes> {
        let line = self.last_line_index();
        let byte_offset = self.last_line_end_column_byte_offset();
        Location(line, byte_offset)
    }
}


// === Validation ===

impl Text {
    /// Check whether the provided line index is valid in this text.
    pub fn validate_line_index(&self, line: Line) -> Result<Line, BoundsError> {
        use BoundsError::*;
        if line < Line(0) {
            Err(TooSmall)
        } else if line > self.last_line_index() {
            Err(TooBig)
        } else {
            Ok(line)
        }
    }

    /// Check whether the provided byte offset is valid in this text.
    pub fn validate_byte_offset(&self, offset: UBytes) -> Result<UBytes, BoundsError> {
        use BoundsError::*;
        if offset < 0.ubytes() {
            Err(TooSmall)
        } else if offset > self.byte_size() {
            Err(TooBig)
        } else {
            Ok(offset)
        }
    }
}



// ===================
// === Conversions ===
// ===================

// === Into Byte Offset ===

impl Text {
    /// Return the offset after the last character of a given line if the line exists.
    pub fn end_byte_offset_of_line_index(&self, line: Line) -> Result<UBytes, BoundsError> {
        self.validate_line_index(line)?;
        let next_line = line + Line(1);
        let next_line_off = self.byte_offset_of_line_index(next_line).ok();
        let next_line_prev = next_line_off.and_then(|t| {
            self.prev_grapheme_offset(t).and_then(|prev1| {
                self.prev_grapheme_offset(prev1).map(|prev2| {
                    let was_rn_seq = self.slice(prev2.value..prev1.value).to_string() == "\r";
                    let off = if was_rn_seq { prev2 } else { prev1 };
                    off
                })
            })
        });
        Ok(next_line_prev.unwrap_or_else(|| self.byte_size()))
    }

    // FIXME: unwraps
    /// Byte length of the given line. Does not include the newline characters.
    pub fn line_byte_length(&self, line: Line) -> UBytes {
        let line_start = self.byte_offset_of_line_index(line).unwrap();
        let line_end = self.end_byte_offset_of_line_index(line).unwrap();
        UBytes::try_from(line_end - line_start).unwrap()
    }

    /// Return the offset after the last character of a given line if the line exists. Snapped to
    /// the closest valid value.
    pub fn end_byte_offset_of_line_index_snapped(&self, line: Line) -> UBytes {
        self.snap_bytes_bounds_result(self.end_byte_offset_of_line_index(line))
    }

    /// The line byte offset. Panics in case the line index was invalid.
    pub fn byte_offset_of_line_index_unchecked(&self, line: Line) -> UBytes {
        self.rope.offset_of_line(line.value).into()
    }

    /// The byte offset of the given line index.
    pub fn byte_offset_of_line_index(&self, line: Line) -> Result<UBytes, BoundsError> {
        self.validate_line_index(line)?;
        Ok(self.byte_offset_of_line_index_unchecked(line))
    }

    /// The byte offset of the given line. Snapped to the closest valid byte offset in case the
    /// line index was invalid.
    pub fn byte_offset_of_line_index_snapped(&self, line: Line) -> UBytes {
        use BoundsError::*;
        match self.byte_offset_of_line_index(line) {
            Ok(offset) => offset,
            Err(TooSmall) => self.first_line_byte_offset(),
            Err(TooBig) => self.last_line_byte_offset(),
        }
    }

    /// Byte offset of the given location.
    pub fn byte_offset_of_location(
        &self,
        location: Location<UBytes>,
    ) -> Result<UBytes, LocationError<UBytes>> {
        let line_offset = self.byte_offset_of_line_index(location.line)?;
        Ok(line_offset + location.offset)
    }

    /// Byte offset of the given location. Snapped to the closest valid value.
    pub fn byte_offset_of_location_snapped(&self, location: Location<UBytes>) -> UBytes {
        let offset = self.byte_offset_of_location(location);
        self.snap_bytes_location_result(offset)
    }

    /// Byte range of the given line.
    pub fn byte_range_of_line_index(
        &self,
        line: Line,
    ) -> Result<std::ops::Range<UBytes>, BoundsError> {
        let start = self.byte_offset_of_line_index(line)?;
        let end = self.end_byte_offset_of_line_index(line)?;
        Ok(start..end)
    }

    /// Byte range of the given line. Snapped to the closest valid value.
    pub fn byte_range_of_line_index_snapped(&self, line: Line) -> std::ops::Range<UBytes> {
        let start = self.byte_offset_of_line_index_snapped(line);
        let end = self.end_byte_offset_of_line_index_snapped(line);
        start..end
    }
}


// === Into Line Index ===

impl Text {
    /// The line of a given byte offset. Panics in case the offset was invalid.
    pub fn line_index_of_byte_offset_unchecked(&self, offset: UBytes) -> Line {
        self.rope.line_of_offset(offset.value).into()
    }

    /// The line index of the given byte offset.
    pub fn line_index_of_byte_offset(&self, offset: UBytes) -> Result<Line, BoundsError> {
        self.validate_byte_offset(offset)?;
        Ok(self.line_index_of_byte_offset_unchecked(offset))
    }

    /// The line index of the given byte offset. Snapped to the closest valid line index in case the
    /// byte offset was invalid.
    pub fn line_index_of_byte_offset_snapped(&self, offset: UBytes) -> Line {
        use BoundsError::*;
        match self.line_index_of_byte_offset(offset) {
            Ok(index) => index,
            Err(TooSmall) => self.first_line_index(),
            Err(TooBig) => self.last_line_index(),
        }
    }
}


// === Into CodePointIndex ===

impl Text {
    // /// The last column number of the given line.
    // pub fn line_end_column(&self, line: Line) -> Result<CodePointIndex, BoundsError> {
    //     let offset = self.end_byte_offset_of_line_index(line)?;
    //     Ok(self.column_of_byte_offset(offset).unwrap())
    // }
    //
    // // fixme: this is not column computing!!!
    // /// The column number of the given byte offset.
    // pub fn column_of_byte_offset(
    //     &self,
    //     tgt_offset: UBytes,
    // ) -> Result<CodePointIndex, LocationError<CodePointIndex>> {
    //     use self::BoundsError::*;
    //     use LocationError::*;
    //     let line_index = self.line_index_of_byte_offset(tgt_offset)?;
    //     let mut offset = self.byte_offset_of_line_index(line_index)?;
    //     let mut code_point_index = 0.code_point_index();
    //     while offset < tgt_offset {
    //         match self.next_codepoint_offset(offset) {
    //             None => return Err(BoundsError(TooBig)),
    //             Some(off) => {
    //                 offset = off;
    //                 code_point_index += 1.code_point_index();
    //             }
    //         }
    //     }
    //     if offset != tgt_offset {
    //         Err(NotClusterBoundary(code_point_index))
    //     } else {
    //         Ok(code_point_index)
    //     }
    // }

    // FIXME: docs
    /// Test
    pub fn line_byte_offset_of_byte_offset(
        &self,
        tgt_offset: UBytes,
    ) -> Result<UBytes, LocationError<CodePointIndex>> {
        let line_index = self.line_index_of_byte_offset(tgt_offset)?;
        let line_offset = self.byte_offset_of_line_index(line_index)?;
        let offset = UBytes::try_from(tgt_offset - line_offset).unwrap();
        Ok(offset)
    }

    // /// The column number of the given byte offset. Snapped to the closest valid
    // /// value. In case the offset points inside of a grapheme cluster, it will be snapped to its
    // /// right side.
    // pub fn column_of_byte_offset_snapped(&self, tgt_offset: UBytes) -> CodePointIndex {
    //     self.snap_column_location_result(self.column_of_byte_offset(tgt_offset))
    // }

    // /// The column from line number and byte offset within the line.
    // pub fn column_of_line_index_and_in_line_byte_offset(
    //     &self,
    //     line: Line,
    //     in_line_offset: UBytes,
    // ) -> Result<CodePointIndex, LocationError<CodePointIndex>> {
    //     let offset = self.byte_offset_of_line_index(line)?;
    //     let tgt_offset = offset + in_line_offset;
    //     let column = self.column_of_byte_offset(tgt_offset)?;
    //     Ok(column)
    // }
    //
    // /// The column from line number and byte offset within the line. Snapped to
    // /// the closest valid value. In case the offset points inside of a grapheme cluster, it will
    // be /// snapped to its right side.
    // pub fn column_of_line_index_and_in_line_byte_offset_snapped(
    //     &self,
    //     line: Line,
    //     in_line_offset: UBytes,
    // ) -> CodePointIndex {
    //     let column = self.column_of_line_index_and_in_line_byte_offset(line, in_line_offset);
    //     self.snap_column_location_result(column)
    // }
}


// === Into Location ===


impl Text {
    /// The location of text end.
    pub fn location_of_text_end(&self) -> Location<UBytes> {
        let lines_count = self.lines(self.byte_range()).count();
        let last_char_off = self.rope.prev_codepoint_offset(self.len());
        let last_char = last_char_off.map(|off| self.rope.slice_to_cow(off..));
        let ends_with_eol = last_char.map_or(false, |ch| ch.starts_with('\n'));
        if ends_with_eol {
            let line: Line = lines_count.into();
            Location(line, UBytes(0))
        } else if lines_count == 0 {
            default()
        } else {
            let line = Line(lines_count - 1);
            let byte_offset = self.end_byte_offset_of_line_index(line).unwrap();
            Location(line, byte_offset)
        }
    }

    /// The location of the provided byte offset.
    pub fn location_of_byte_offset(&self, offset: UBytes) -> Result<Location<UBytes>, BoundsError> {
        let line = self.line_index_of_byte_offset(offset)?;
        let line_offset = offset - self.byte_offset_of_line_index(line).unwrap(); // fixme unwrap
        let line_offset = UBytes::try_from(line_offset).unwrap_or_else(|_| {
            error!("Internal error, wrong line byte offset.");
            UBytes(0)
        });
        let byte_offset = UBytes::try_from(offset - line_offset).unwrap();
        Ok(Location(line, byte_offset))
    }

    /// The location of the provided byte offset. Snapped to the closest valid
    /// value.
    pub fn location_of_byte_offset_snapped(&self, offset: UBytes) -> Location<UBytes> {
        use BoundsError::*;
        match self.location_of_byte_offset(offset) {
            Ok(location) => location,
            Err(TooSmall) => self.first_line_start_location(),
            Err(TooBig) => self.last_line_end_location(),
        }
    }
}



// ==============
// === Errors ===
// ==============

/// Error indicating the usage of incorrect line index or byte offset - negative, or bigger than the
/// size of the text.
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub enum BoundsError {
    TooSmall,
    TooBig,
}

/// Error indicating the usage of incorrect location in the text.
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub enum LocationError<T> {
    BoundsError(BoundsError),
    LineTooShort(T),
    NotClusterBoundary(T),
}

impl<T> From<BoundsError> for LocationError<T> {
    fn from(err: BoundsError) -> Self {
        Self::BoundsError(err)
    }
}

impl Text {
    // /// Snaps the `LocationError<CodePointIndex>` to the closest valid column.
    // pub fn snap_column_location_error(&self, err: LocationError<CodePointIndex>) ->
    // CodePointIndex {     use self::BoundsError::*;
    //     use LocationError::*;
    //     match err {
    //         BoundsError(TooSmall) => 0.code_point_index(),
    //         BoundsError(TooBig) => self.last_line_end_column(),
    //         LineTooShort(column) => column,
    //         NotClusterBoundary(column) => column,
    //     }
    // }

    /// Snaps the `LocationError<UBytes>` to the closest valid byte offset.
    pub fn snap_bytes_location_error(&self, err: LocationError<UBytes>) -> UBytes {
        use self::BoundsError::*;
        use LocationError::*;
        match err {
            BoundsError(TooSmall) => 0.ubytes(),
            BoundsError(TooBig) => self.last_line_end_byte_offset(),
            LineTooShort(offset) => offset,
            NotClusterBoundary(offset) => offset,
        }
    }

    /// Snaps the `BoundsError` to the closest valid byte offset.
    pub fn snap_bytes_bounds_error(&self, err: BoundsError) -> UBytes {
        use self::BoundsError::*;
        match err {
            TooSmall => 0.ubytes(),
            TooBig => self.last_line_end_byte_offset(),
        }
    }

    // /// Snaps the `LocationResult<CodePointIndex>` to the closest valid column.
    // pub fn snap_column_location_result(
    //     &self,
    //     result: Result<CodePointIndex, LocationError<CodePointIndex>>,
    // ) -> CodePointIndex {
    //     match result {
    //         Ok(column) => column,
    //         Err(err) => self.snap_column_location_error(err),
    //     }
    // }

    /// Snaps the `LocationResult<UBytes>` to the closest valid byte offset.
    pub fn snap_bytes_location_result(
        &self,
        result: Result<UBytes, LocationError<UBytes>>,
    ) -> UBytes {
        match result {
            Ok(bytes) => bytes,
            Err(err) => self.snap_bytes_location_error(err),
        }
    }

    /// Snaps the `Result<UBytes,BoundsError>` to the closest valid byte offset.
    pub fn snap_bytes_bounds_result(&self, result: Result<UBytes, BoundsError>) -> UBytes {
        match result {
            Ok(bytes) => bytes,
            Err(err) => self.snap_bytes_bounds_error(err),
        }
    }
}


// === Common Prefix and Suffix ===

/// The return value of [`Text::common_prefix_and_suffix`] function.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CommonPrefixAndSuffix {
    pub prefix: UBytes,
    pub suffix: UBytes,
}

impl Text {
    /// Returns the length in bytes of common prefix and suffix.
    ///
    /// The prefix and suffix lengths does not overlap, so the sum of their length will not exceed
    /// the length of both texts.
    pub fn common_prefix_and_suffix(&self, other: &Text) -> CommonPrefixAndSuffix {
        let mut scanner = xi_rope::compare::RopeScanner::new(&self.rope, &other.rope);
        let (prefix, suffix) = scanner.find_min_diff_range();
        CommonPrefixAndSuffix { prefix: prefix.into(), suffix: suffix.into() }
    }
}

// === Display ===

impl Display for Text {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.rope, f)
    }
}


// ===================
// === Conversions ===
// ===================

impl From<Rope> for Text {
    fn from(t: Rope) -> Self {
        Self { rope: t }
    }
}

impl From<&Rope> for Text {
    fn from(t: &Rope) -> Self {
        t.clone().into()
    }
}

impl From<&str> for Text {
    fn from(t: &str) -> Self {
        Self { rope: t.into() }
    }
}

impl From<String> for Text {
    fn from(t: String) -> Self {
        Self { rope: t.into() }
    }
}

impl From<&String> for Text {
    fn from(t: &String) -> Self {
        Self { rope: t.into() }
    }
}

impl From<&&String> for Text {
    fn from(t: &&String) -> Self {
        (*t).into()
    }
}

impl From<ImString> for Text {
    fn from(t: ImString) -> Self {
        Self { rope: t.into() }
    }
}

impl From<&ImString> for Text {
    fn from(t: &ImString) -> Self {
        Self { rope: t.into() }
    }
}

impl From<&&ImString> for Text {
    fn from(t: &&ImString) -> Self {
        (*t).into()
    }
}

impl From<&&str> for Text {
    fn from(t: &&str) -> Self {
        (*t).into()
    }
}

impl From<Text> for String {
    fn from(t: Text) -> Self {
        t.rope.into()
    }
}

impl From<&Text> for String {
    fn from(t: &Text) -> Self {
        t.clone().into()
    }
}

impl From<&&Text> for String {
    fn from(t: &&Text) -> Self {
        (*t).into()
    }
}



// ================
// === TextCell ===
// ================

/// Internally mutable version of `Text`.
#[derive(Debug, Clone, Default, Deref)]
#[allow(missing_docs)]
pub struct TextCell {
    cell: RefCell<Text>,
}

impl TextCell {
    /// Getter of the current value of the cell.
    pub fn get(&self) -> Text {
        self.cell.borrow().clone()
    }

    /// Setter of the value of the cell.
    pub fn set(&self, new_text: impl Into<Text>) {
        let new_text = new_text.into();
        *self.cell.borrow_mut() = new_text;
    }

    /// Get all lines in the provided range as strings.
    pub fn lines_vec(&self, range: std::ops::Range<UBytes>) -> Vec<String> {
        let rope_range = range.start.value..range.end.value;
        let mut lines = self.cell.borrow().lines(rope_range).map(|t| t.into()).collect_vec();
        if lines.is_empty() {
            // Rope returns `[]` if the line is empty.
            lines.push("".into())
        }
        lines
    }
}

/// See docs in `Text`.
#[allow(missing_docs)]
impl TextCell {
    pub fn new() -> Self {
        default()
    }

    pub fn is_empty(&self) -> bool {
        self.cell.borrow().is_empty()
    }

    pub fn sub(&self, range: impl RangeBounds) -> Text {
        self.cell.borrow().sub(range)
    }

    pub fn grapheme_count(&self) -> usize {
        self.cell.borrow().grapheme_count()
    }

    pub fn byte_size(&self) -> UBytes {
        self.cell.borrow().byte_size()
    }

    pub fn byte_range(&self) -> Range<UBytes> {
        self.cell.borrow().byte_range()
    }

    pub fn crop_byte_range(&self, range: impl RangeBounds) -> Range<UBytes> {
        self.cell.borrow().crop_byte_range(range)
    }

    pub fn snap_location(&self, location: Location<UBytes>) -> Location<UBytes> {
        self.cell.borrow().snap_location(location)
    }

    pub fn next_grapheme_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.cell.borrow().next_grapheme_offset(offset)
    }

    pub fn prev_grapheme_offset(&self, offset: UBytes) -> Option<UBytes> {
        self.cell.borrow().prev_grapheme_offset(offset)
    }

    pub fn replace(&self, range: impl RangeBounds, text: impl Into<Text>) {
        self.cell.borrow_mut().replace(range, text)
    }

    pub fn first_line_index(&self) -> Line {
        self.cell.borrow().first_line_index()
    }

    pub fn first_line_byte_offset(&self) -> UBytes {
        self.cell.borrow().first_line_byte_offset()
    }

    pub fn first_line_start_column(&self) -> CodePointIndex {
        self.cell.borrow().first_line_start_column()
    }

    pub fn first_line_start_location(&self) -> Location<UBytes> {
        self.cell.borrow().first_line_start_location()
    }

    pub fn last_line_index(&self) -> Line {
        self.cell.borrow().last_line_index()
    }

    pub fn last_line_byte_offset(&self) -> UBytes {
        self.cell.borrow().last_line_byte_offset()
    }

    pub fn last_line_start_column(&self) -> CodePointIndex {
        self.cell.borrow().last_line_start_column()
    }

    pub fn last_line_start_location(&self) -> Location<UBytes> {
        self.cell.borrow().last_line_start_location()
    }

    // pub fn last_line_end_column(&self) -> CodePointIndex {
    //     self.cell.borrow().last_line_end_column()
    // }

    pub fn last_line_end_byte_offset(&self) -> UBytes {
        self.cell.borrow().last_line_end_byte_offset()
    }

    pub fn last_line_end_location(&self) -> Location<UBytes> {
        self.cell.borrow().last_line_end_location()
    }

    pub fn validate_line_index(&self, line: Line) -> Result<Line, BoundsError> {
        self.cell.borrow().validate_line_index(line)
    }

    pub fn validate_byte_offset(&self, offset: UBytes) -> Result<UBytes, BoundsError> {
        self.cell.borrow().validate_byte_offset(offset)
    }

    pub fn end_byte_offset_of_line_index(&self, line: Line) -> Result<UBytes, BoundsError> {
        self.cell.borrow().end_byte_offset_of_line_index(line)
    }

    pub fn line_byte_length(&self, line: Line) -> UBytes {
        self.cell.borrow().line_byte_length(line)
    }

    pub fn end_byte_offset_of_line_index_snapped(&self, line: Line) -> UBytes {
        self.cell.borrow().end_byte_offset_of_line_index_snapped(line)
    }

    pub fn byte_offset_of_line_index_unchecked(&self, line: Line) -> UBytes {
        self.cell.borrow().byte_offset_of_line_index_unchecked(line)
    }

    pub fn byte_offset_of_line_index(&self, line: Line) -> Result<UBytes, BoundsError> {
        self.cell.borrow().byte_offset_of_line_index(line)
    }

    pub fn byte_offset_of_line_index_snapped(&self, line: Line) -> UBytes {
        self.cell.borrow().byte_offset_of_line_index_snapped(line)
    }

    pub fn byte_offset_of_location(
        &self,
        location: Location<UBytes>,
    ) -> Result<UBytes, LocationError<UBytes>> {
        self.cell.borrow().byte_offset_of_location(location)
    }

    pub fn byte_offset_of_location_snapped(&self, location: Location<UBytes>) -> UBytes {
        self.cell.borrow().byte_offset_of_location_snapped(location)
    }

    pub fn byte_range_of_line_index(
        &self,
        line: Line,
    ) -> Result<std::ops::Range<UBytes>, BoundsError> {
        self.cell.borrow().byte_range_of_line_index(line)
    }

    pub fn byte_range_of_line_index_snapped(&self, line: Line) -> std::ops::Range<UBytes> {
        self.cell.borrow().byte_range_of_line_index_snapped(line)
    }

    pub fn line_index_of_byte_offset_unchecked(&self, offset: UBytes) -> Line {
        self.cell.borrow().line_index_of_byte_offset_unchecked(offset)
    }

    pub fn line_index_of_byte_offset(&self, offset: UBytes) -> Result<Line, BoundsError> {
        self.cell.borrow().line_index_of_byte_offset(offset)
    }

    pub fn line_index_of_byte_offset_snapped(&self, offset: UBytes) -> Line {
        self.cell.borrow().line_index_of_byte_offset_snapped(offset)
    }

    // pub fn line_end_column(&self, line: Line) -> Result<CodePointIndex, BoundsError> {
    //     self.cell.borrow().line_end_column(line)
    // }

    // pub fn column_of_byte_offset(
    //     &self,
    //     tgt_offset: UBytes,
    // ) -> Result<CodePointIndex, LocationError<CodePointIndex>> {
    //     self.cell.borrow().column_of_byte_offset(tgt_offset)
    // }

    // pub fn column_of_byte_offset_snapped(&self, tgt_offset: UBytes) -> CodePointIndex {
    //     self.cell.borrow().column_of_byte_offset_snapped(tgt_offset)
    // }

    // pub fn column_of_line_index_and_in_line_byte_offset(
    //     &self,
    //     line: Line,
    //     in_line_offset: UBytes,
    // ) -> Result<CodePointIndex, LocationError<CodePointIndex>> {
    //     self.cell.borrow().column_of_line_index_and_in_line_byte_offset(line, in_line_offset)
    // }

    // pub fn column_of_line_index_and_in_line_byte_offset_snapped(
    //     &self,
    //     line: Line,
    //     in_line_offset: UBytes,
    // ) -> CodePointIndex {
    //     self.cell
    //         .borrow()
    //         .column_of_line_index_and_in_line_byte_offset_snapped(line, in_line_offset)
    // }

    pub fn location_of_byte_offset(&self, offset: UBytes) -> Result<Location<UBytes>, BoundsError> {
        self.cell.borrow().location_of_byte_offset(offset)
    }

    pub fn location_of_byte_offset_snapped(&self, offset: UBytes) -> Location<UBytes> {
        self.cell.borrow().location_of_byte_offset_snapped(offset)
    }
}



// ==============
// === Change ===
// ==============

/// A single change done to the text content.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Change<Metric = UBytes, Str = Text> {
    /// Range of old text being replaced.
    pub range: Range<Metric>,
    /// The text inserted in place of `range`.
    pub text:  Str,
}


impl<Metric, String> Change<Metric, String> {
    /// Create a change being an insert of the `text` at given `offset` (no text will be removed).
    pub fn inserted(offset: Metric, text: String) -> Self
    where Metric: Copy {
        Self { range: Range::new(offset, offset), text }
    }

    /// Return new [`Change`] with copied range and a reference to self's string.
    pub fn as_ref(&self) -> Change<Metric, &String>
    where Metric: Copy {
        Change { range: self.range, text: &self.text }
    }
}


// === Applying Change ===

impl<S: AsRef<str>> Change<UBytes, S> {
    /// Apply the change on the given string.
    pub fn apply(&self, target: &mut String) -> Result<(), BoundsError> {
        let start_byte = self.range.start.value;
        let end_byte = self.range.end.value;
        target.replace_range(start_byte..end_byte, self.text.as_ref());
        Ok(())
    }

    /// Return a new string being a `target` with this change applied.
    pub fn applied(&self, target: &str) -> Result<String, BoundsError> {
        let mut string = target.to_owned();
        self.apply(&mut string)?;
        Ok(string)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn location_of_text_end() {
        struct Case {
            text:     &'static str,
            expected: (usize, usize), // Line and column
        }

        impl Case {
            fn run(&self) {
                let text: Text = self.text.into();
                let (exp_line, exp_column) = self.expected;
                let expected =
                    Location { line: exp_line.into(), offset: UBytes(exp_column.into()) };
                let result = text.location_of_text_end();
                assert_eq!(result, expected, "Wrong text end location in case \"{}\"", text);
            }
        }

        let cases = &[
            Case { text: "", expected: (0, 0) },
            Case { text: "single line", expected: (0, 11) },
            Case { text: "single line with eol\n", expected: (1, 0) },
            Case { text: "\nMany\nLines", expected: (2, 5) },
            Case { text: "Many\nLines\nwith eol\n", expected: (3, 0) },
        ];

        for case in cases {
            case.run()
        }
    }
}
