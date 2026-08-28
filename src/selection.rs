#[derive(Debug)]
pub struct Selection<T> {
    items: Vec<SelectionItem<T>>,
    highlighted: Option<usize>,
}

impl<T> Selection<T> {
    pub fn new(items: impl IntoIterator<Item = T>) -> Self {
        let items: Vec<_> = items.into_iter().map(SelectionItem::new).collect();
        let highlighted = (!items.is_empty()).then_some(0);

        Self { items, highlighted }
    }

    pub fn items(&self) -> impl Iterator<Item = &SelectionItem<T>> {
        self.items.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().map(|item| &mut item.value)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    pub fn highlight_previous(&mut self) {
        if let Some(highlighted) = self.highlighted.as_mut() {
            *highlighted = highlighted.saturating_sub(1);
        }
    }

    pub fn highlight_next(&mut self) {
        if let Some(highlighted) = self.highlighted.as_mut() {
            *highlighted = highlighted
                .saturating_add(1)
                .min(self.items.len().saturating_sub(1));
        }
    }

    pub fn toggle_highlighted(&mut self) {
        if let Some(item) = self
            .highlighted
            .and_then(|highlighted| self.items.get_mut(highlighted))
        {
            item.selected = !item.selected;
        }
    }

    pub fn toggle_all(&mut self) {
        let selected = !self.items.iter().all(SelectionItem::is_selected);

        for item in &mut self.items {
            item.selected = selected;
        }
    }
}

#[derive(Debug)]
pub struct SelectionItem<T> {
    value: T,
    selected: bool,
}

impl<T> SelectionItem<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            selected: true,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}
