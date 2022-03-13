pub mod mock {
    use crate::AnyMsg;
    use crate::primitives::alphabet::mock::ALPHABET;
    use crate::widgets::editor_view::msg::EditorViewMsg;
    use crate::widgets::fuzzy_search::helpers::is_substring;
    use crate::widgets::fuzzy_search::item_provider::{Item, ItemsProvider};

    pub struct MockItemProvider {
        num_items: usize,
        items: Vec<String>,
    }

    impl MockItemProvider {
        pub fn new(num_items: usize) -> Self {
            let mut items: Vec<String> = vec![];
            
            for i in 0..num_items {
                let mut idx = i;
                let mut item = String::default();

                loop {
                    if !item.is_empty() {
                        item += " ";
                    }

                    item += ALPHABET[idx % ALPHABET.len()];
                    idx /= ALPHABET.len();
                    if idx == 0 {
                        break;
                    }
                }

                items.push(item);
            }

            MockItemProvider {
                num_items,
                items,
            }
        }
    }

    impl Item for String {
        fn display_name(&self) -> &str {
            self.as_str()
        }

        fn on_hit(&self) -> Box<dyn AnyMsg> {
            Box::new(EditorViewMsg::FuzzyClose)
        }
    }

    impl ItemsProvider for MockItemProvider {
        fn context_name(&self) -> &str {
            "mock"
        }

        fn items<'a>(&'a self, query: String) -> Box<dyn Iterator<Item=&'a dyn Item> + '_> {
            Box::new(self.items.iter().filter(move |t| is_substring(t, &query)).map(|f| f as &dyn Item))
        }
    }
}