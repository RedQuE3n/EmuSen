//! Blocks by physical word address, in pages of 1,024 allocated when code first runs there. C#'s `BlockCache`; see Mars_Native.md §5.8.

use crate::cpu::blocks::Block;

const PAGE_WORDS: usize = 1024;

/// One page's entries, a slot a word.
type Page = Box<[Option<Box<Block>>]>;

#[derive(Default)]
pub struct Cache {
    pages: Vec<Option<Page>>,
    pub live: usize,
}

impl Cache {
    #[inline(always)]
    pub fn get(&self, physical: u32) -> Option<&Block> {
        let word = (physical >> 2) as usize;
        self.pages.get(word / PAGE_WORDS)?.as_ref()?[word % PAGE_WORDS].as_deref()
    }

    #[inline(always)]
    pub fn get_mut(&mut self, physical: u32) -> Option<&mut Block> {
        let word = (physical >> 2) as usize;
        self.pages.get_mut(word / PAGE_WORDS)?.as_mut()?[word % PAGE_WORDS].as_deref_mut()
    }

    /// Places a block at its start, replacing any there.
    pub fn place(&mut self, block: Block) -> &mut Block {
        let word = (block.start >> 2) as usize;
        let (page, at) = (word / PAGE_WORDS, word % PAGE_WORDS);
        if self.pages.len() <= page {
            self.pages.resize_with(page + 1, || None);
        }
        let slots = self.pages[page].get_or_insert_with(|| (0..PAGE_WORDS).map(|_| None).collect());
        if slots[at].is_none() {
            self.live += 1;
        }
        slots[at].insert(Box::new(block))
    }

    pub fn clear(&mut self) {
        self.pages.clear();
        self.live = 0;
    }
}
