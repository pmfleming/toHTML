use super::{Block, Caret, Doc, Table, TableCell, TableRow};

impl Doc {
    pub fn insert_table_after(&mut self, caret: &mut Caret, rows: usize, cols: usize) {
        let rows = rows.clamp(1, 20);
        let cols = cols.clamp(1, 12);
        let table = Table {
            caption: None,
            rows: (0..rows)
                .map(|r| TableRow {
                    cells: (0..cols)
                        .map(|_| TableCell {
                            header: r == 0,
                            colspan: 1,
                            rowspan: 1,
                            align: None,
                            content: Vec::new(),
                        })
                        .collect(),
                })
                .collect(),
        };
        let pos = caret.block.saturating_add(1).min(self.blocks.len());
        self.blocks.insert(pos, Block::Table(table));
        caret.block = pos;
        caret.table_row = 0;
        caret.table_col = 0;
        caret.char = 0;
    }

    pub fn add_table_row(&mut self, caret: &mut Caret) -> bool {
        let Some(Block::Table(table)) = self.blocks.get_mut(caret.block) else {
            return false;
        };
        let cols = table
            .rows
            .first()
            .map(|r| r.cells.len())
            .unwrap_or(1)
            .max(1);
        let row = TableRow {
            cells: (0..cols)
                .map(|_| TableCell {
                    colspan: 1,
                    rowspan: 1,
                    ..Default::default()
                })
                .collect(),
        };
        let insert_at = caret.table_row.saturating_add(1).min(table.rows.len());
        table.rows.insert(insert_at, row);
        caret.table_row = insert_at;
        caret.table_col = caret.table_col.min(cols - 1);
        caret.char = 0;
        true
    }

    pub fn add_table_col(&mut self, caret: &mut Caret) -> bool {
        let Some(Block::Table(table)) = self.blocks.get_mut(caret.block) else {
            return false;
        };
        for (row_idx, row) in table.rows.iter_mut().enumerate() {
            let header = row_idx == 0;
            let insert_at = caret.table_col.saturating_add(1).min(row.cells.len());
            row.cells.insert(
                insert_at,
                TableCell {
                    header,
                    colspan: 1,
                    rowspan: 1,
                    ..Default::default()
                },
            );
        }
        caret.table_col += 1;
        caret.char = 0;
        true
    }
}
