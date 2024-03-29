use tui_tree_widget::{TreeItem, TreeState};

pub struct StatefulTree<'a> {
    pub state: TreeState,
    pub items: Vec<TreeItem<'a>>,
}

impl<'a> StatefulTree<'a> {
    pub fn with_items(items: Vec<TreeItem<'a>>) -> Self {
        Self {
            state: TreeState::default(),
            items,
        }
    }

    #[allow(dead_code)]
    pub fn first(&mut self) {
        self.state.select_first();
    }

    #[allow(dead_code)]
    pub fn last(&mut self) {
        self.state.select_last(&self.items);
    }

    pub fn down(&mut self) {
        self.state.key_down(&self.items);
    }

    pub fn up(&mut self) {
        self.state.key_up(&self.items);
    }

    pub fn left(&mut self) {
        self.state.key_left();
    }

    pub fn right(&mut self) {
        self.state.key_right();
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self) {
        self.state.toggle_selected();
    }
}
