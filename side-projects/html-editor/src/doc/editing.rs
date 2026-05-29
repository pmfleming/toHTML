use super::{Block, Caret, Doc, InlineStyle, StyledChar, Table};

impl Doc {
    pub fn insert_text(&mut self, caret: &mut Caret, text: &str, style: &InlineStyle) -> bool {
        if text.is_empty() {
            return false;
        }
        if self.blocks.is_empty() {
            self.blocks.push(Block::Paragraph(Vec::new()));
            caret.block = 0;
            caret.char = 0;
        }
        let mut inserted = false;
        if let Some(runs) = self.current_inline_mut(caret) {
            for c in text.chars() {
                runs.insert(
                    caret.char,
                    StyledChar {
                        ch: c,
                        style: style.clone(),
                    },
                );
                caret.char += 1;
                inserted = true;
            }
        }
        inserted
    }

    pub fn backspace(&mut self, caret: &mut Caret) -> bool {
        if caret.block >= self.blocks.len() {
            return false;
        }
        if caret.char == 0 {
            if caret.block == 0 {
                return false;
            }
            if !self.can_merge_inline_blocks(caret.block - 1, caret.block) {
                return false;
            }
            let cur = self.blocks.remove(caret.block);
            caret.block -= 1;
            let new_char = self.blocks[caret.block].len();
            let items = cur.inline().cloned().unwrap_or_default();
            if let Some(prev) = self.blocks[caret.block].inline_mut() {
                prev.extend(items);
            }
            caret.char = new_char;
            caret.table_row = 0;
            caret.table_col = 0;
            return true;
        }
        if let Some(runs) = self.current_inline_mut(caret) {
            if caret.char > 0 && caret.char <= runs.len() {
                runs.remove(caret.char - 1);
                caret.char -= 1;
                return true;
            }
        }
        false
    }

    pub fn delete_forward(&mut self, caret: &mut Caret) -> bool {
        if caret.block >= self.blocks.len() {
            return false;
        }
        let len = self.current_len(caret);
        if caret.char < len {
            if let Some(runs) = self.current_inline_mut(caret) {
                runs.remove(caret.char);
                return true;
            }
        } else if caret.block + 1 < self.blocks.len() {
            if !self.can_merge_inline_blocks(caret.block, caret.block + 1) {
                return false;
            }
            let next = self.blocks.remove(caret.block + 1);
            let items = next.inline().cloned().unwrap_or_default();
            if let Some(cur) = self.blocks[caret.block].inline_mut() {
                cur.extend(items);
                return true;
            }
        }
        false
    }

    pub fn split_block(&mut self, caret: &mut Caret) -> bool {
        if self.blocks.is_empty() {
            self.blocks.push(Block::Paragraph(Vec::new()));
            caret.block = 0;
            caret.char = 0;
            return true;
        }
        if caret.block >= self.blocks.len() {
            return false;
        }
        let cur_kind = self.blocks[caret.block].clone();
        if matches!(cur_kind, Block::Table(_)) {
            let style = self.current_style_at(caret);
            return self.insert_text(caret, "\n", &style);
        }
        // Empty list item -> exit list.
        if cur_kind.is_list_item() && cur_kind.len() == 0 {
            self.blocks[caret.block] = Block::Paragraph(Vec::new());
            caret.char = 0;
            return true;
        }
        // Empty blockquote -> escape.
        if matches!(cur_kind, Block::Blockquote(_)) && cur_kind.len() == 0 {
            self.blocks[caret.block] = Block::Paragraph(Vec::new());
            caret.char = 0;
            return true;
        }
        let tail: Vec<StyledChar> = {
            let Some(runs) = self.blocks[caret.block].inline_mut() else {
                return false;
            };
            runs.split_off(caret.char.min(runs.len()))
        };
        let new_block = match &cur_kind {
            Block::Heading(_, _) => Block::Paragraph(tail), // Enter in heading → paragraph
            Block::Bullet(_) => Block::Bullet(tail),
            Block::Numbered(_) => Block::Numbered(tail),
            Block::Blockquote(_) => Block::Blockquote(tail),
            Block::Pre(_) => Block::Pre(tail),
            _ => Block::Paragraph(tail),
        };
        self.blocks.insert(caret.block + 1, new_block);
        caret.block += 1;
        caret.char = 0;
        true
    }

    pub fn move_left(&self, caret: &mut Caret) -> bool {
        if caret.char > 0 {
            caret.char -= 1;
            return true;
        }
        if self.move_to_previous_table_cell_end(caret) {
            return true;
        }
        if caret.block > 0 {
            self.set_caret_to_block_end(caret, caret.block - 1);
            return true;
        }
        false
    }

    pub fn move_right(&self, caret: &mut Caret) -> bool {
        let len = self.current_len(caret);
        if caret.char < len {
            caret.char += 1;
            return true;
        }
        if self.move_table_cell(caret, 1) {
            return true;
        }
        if caret.block + 1 < self.blocks.len() {
            self.set_caret_to_block_start(caret, caret.block + 1);
            return true;
        }
        false
    }

    pub fn transform_block_to<F: FnOnce(Vec<StyledChar>) -> Block>(&mut self, idx: usize, mk: F) {
        let Some(b) = self.blocks.get_mut(idx) else {
            return;
        };
        let runs = if let Some(v) = b.inline_mut() {
            std::mem::take(v)
        } else {
            return;
        };
        *b = mk(runs);
    }

    pub fn apply_style_range(
        &mut self,
        lo: &Caret,
        hi: &Caret,
        toggle: impl Fn(&mut InlineStyle),
    ) -> bool {
        if lo.block == hi.block && lo.table_row == hi.table_row && lo.table_col == hi.table_col {
            let Some(runs) = self.current_inline_mut(lo) else {
                return false;
            };
            let start = lo.char.min(runs.len());
            let end = hi.char.min(runs.len());
            if start >= end {
                return false;
            }
            for c in &mut runs[start..end] {
                toggle(&mut c.style);
            }
            return true;
        }

        let mut changed = false;
        for bi in lo.block..=hi.block {
            let Some(b) = self.blocks.get_mut(bi) else {
                continue;
            };
            let Some(runs) = b.inline_mut() else {
                continue;
            };
            let len = runs.len();
            let start = if bi == lo.block { lo.char.min(len) } else { 0 };
            let end = if bi == hi.block {
                hi.char.min(len)
            } else {
                len
            };
            for i in start..end {
                if let Some(c) = runs.get_mut(i) {
                    toggle(&mut c.style);
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn current_style_at(&self, caret: &Caret) -> InlineStyle {
        let Some(runs) = self.current_inline(caret) else {
            return InlineStyle::default();
        };
        if runs.is_empty() {
            return InlineStyle::default();
        }
        let idx = caret.char.saturating_sub(1).min(runs.len() - 1);
        runs[idx].style.clone()
    }

    pub fn current_len(&self, caret: &Caret) -> usize {
        self.current_inline(caret)
            .map(|runs| runs.len())
            .unwrap_or(0)
    }

    pub fn current_inline(&self, caret: &Caret) -> Option<&Vec<StyledChar>> {
        match self.blocks.get(caret.block)? {
            Block::Table(table) => table
                .rows
                .get(caret.table_row)?
                .cells
                .get(caret.table_col)
                .map(|cell| &cell.content),
            block => block.inline(),
        }
    }

    pub fn current_inline_mut(&mut self, caret: &Caret) -> Option<&mut Vec<StyledChar>> {
        match self.blocks.get_mut(caret.block)? {
            Block::Table(table) => table
                .rows
                .get_mut(caret.table_row)?
                .cells
                .get_mut(caret.table_col)
                .map(|cell| &mut cell.content),
            block => block.inline_mut(),
        }
    }

    pub fn move_to_block_end(&mut self, caret: &mut Caret, block: usize) {
        caret.block = block.min(self.blocks.len().saturating_sub(1));
        caret.table_row = 0;
        caret.table_col = 0;
        self.clamp_caret(caret);
        caret.char = self.current_len(caret);
    }

    pub fn move_to_table_cell(&mut self, caret: &mut Caret, block: usize, row: usize, col: usize) {
        caret.block = block.min(self.blocks.len().saturating_sub(1));
        caret.table_row = row;
        caret.table_col = col;
        self.clamp_caret(caret);
        caret.char = self.current_len(caret);
    }

    pub fn move_table_cell(&self, caret: &mut Caret, delta: isize) -> bool {
        let Some(Block::Table(table)) = self.blocks.get(caret.block) else {
            return false;
        };
        let cells = table_cell_positions(table);
        let Some(current) = cells
            .iter()
            .position(|&(row, col)| row == caret.table_row && col == caret.table_col)
        else {
            return false;
        };
        let next = current as isize + delta;
        if next < 0 || next >= cells.len() as isize {
            return false;
        }
        let (row, col) = cells[next as usize];
        caret.table_row = row;
        caret.table_col = col;
        caret.char = 0;
        true
    }

    pub fn move_table_row(&self, caret: &mut Caret, delta: isize) -> bool {
        let Some(Block::Table(table)) = self.blocks.get(caret.block) else {
            return false;
        };
        let next = caret.table_row as isize + delta;
        if next < 0 || next >= table.rows.len() as isize {
            return false;
        }
        let next = next as usize;
        let Some(row) = table.rows.get(next) else {
            return false;
        };
        if row.cells.is_empty() {
            return false;
        }
        let desired_char = caret.char;
        caret.table_row = next;
        caret.table_col = caret.table_col.min(row.cells.len() - 1);
        caret.char = desired_char.min(self.current_len(caret));
        true
    }

    fn can_merge_inline_blocks(&self, left: usize, right: usize) -> bool {
        self.blocks.get(left).and_then(Block::inline).is_some()
            && self.blocks.get(right).and_then(Block::inline).is_some()
    }

    fn move_to_previous_table_cell_end(&self, caret: &mut Caret) -> bool {
        let Some(Block::Table(table)) = self.blocks.get(caret.block) else {
            return false;
        };
        if caret.table_col > 0 {
            caret.table_col -= 1;
            caret.char = self.current_len(caret);
            return true;
        }
        if caret.table_row == 0 {
            return false;
        }
        let previous_row = caret.table_row - 1;
        let Some(row) = table.rows.get(previous_row) else {
            return false;
        };
        if row.cells.is_empty() {
            return false;
        }
        caret.table_row = previous_row;
        caret.table_col = row.cells.len() - 1;
        caret.char = self.current_len(caret);
        true
    }

    fn set_caret_to_block_start(&self, caret: &mut Caret, block: usize) {
        caret.block = block;
        caret.table_row = 0;
        caret.table_col = 0;
        caret.char = 0;
    }

    fn set_caret_to_block_end(&self, caret: &mut Caret, block: usize) {
        caret.block = block;
        match self.blocks.get(block) {
            Some(Block::Table(table)) => {
                if let Some((row, col)) = table_cell_positions(table).last().copied() {
                    caret.table_row = row;
                    caret.table_col = col;
                } else {
                    caret.table_row = 0;
                    caret.table_col = 0;
                }
            }
            _ => {
                caret.table_row = 0;
                caret.table_col = 0;
            }
        }
        caret.char = self.current_len(caret);
    }
}

fn table_cell_positions(table: &Table) -> Vec<(usize, usize)> {
    table
        .rows
        .iter()
        .enumerate()
        .flat_map(|(row_idx, row)| (0..row.cells.len()).map(move |col_idx| (row_idx, col_idx)))
        .collect()
}
