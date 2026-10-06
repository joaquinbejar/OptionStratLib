//! The titled table sections every terminal report is made of, and the two
//! ways to emit them: as plain text, or to stdout with colours.

use crate::error::GraphError;
use prettytable::{Attr, Cell, Row, Table, color, format};
use std::io::{self, Write};

/// One table of a report, with the line printed above it.
pub(super) struct Section {
    title: Option<String>,
    table: Table,
}

impl Section {
    /// A section printing `title` on its own line above `table`.
    #[must_use]
    pub(super) fn titled(title: &str, table: Table) -> Self {
        Self {
            title: Some(title.to_string()),
            table,
        }
    }

    /// A section with no line above its table.
    #[must_use]
    pub(super) fn untitled(table: Table) -> Self {
        Self { title: None, table }
    }
}

/// An empty table drawn with box characters, the style of every report.
#[must_use]
pub(super) fn boxed_table() -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table
}

/// A header row, each cell in `colour`.
#[must_use]
pub(super) fn header_row(headers: &[&str], colour: color::Color) -> Row {
    Row::new(
        headers
            .iter()
            .map(|header| Cell::new(header).with_style(Attr::ForegroundColor(colour)))
            .collect(),
    )
}

/// The sections as plain text, without ANSI escapes: each title on its own
/// line, then its table.
#[must_use]
pub(super) fn render(sections: &[Section]) -> String {
    let mut out = String::new();
    for section in sections {
        if let Some(title) = &section.title {
            out.push_str(title);
            out.push('\n');
        }
        out.push_str(&section.table.to_string());
    }
    out
}

/// Writes the sections to stdout, coloured when stdout is a terminal.
///
/// # Errors
///
/// Returns [`GraphError::Io`] when stdout cannot be written.
#[inline(never)]
pub(super) fn print(sections: &[Section]) -> Result<(), GraphError> {
    for section in sections {
        if let Some(title) = &section.title {
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{title}")?;
            stdout.flush()?;
        }
        section.table.print_tty(false)?;
    }
    Ok(())
}
