use streaming_iterator::StreamingIterator;

pub struct GraphemeLinesStreamingIterator<'a> {
    grapheme_it: Box<dyn Iterator<Item=&'a str> + 'a>,
    s: String,
    done: bool,
}

impl<'a> GraphemeLinesStreamingIterator<'a> {
    pub fn new(grapheme_it: Box<dyn Iterator<Item=&'a str> + 'a>) -> Self {
        Self {
            grapheme_it,
            s: "".to_string(),
            done: false,
        }
    }
}

impl<'a> StreamingIterator for GraphemeLinesStreamingIterator<'a> {
    type Item = String;

    fn advance(&mut self) {
        self.s.clear();

        let mut is_empty = true;

        while let Some(grapheme) = self.grapheme_it.next() {
            if grapheme.contains("\n") {
                break;
            }

            self.s += grapheme;
            is_empty = false;
        }

        if is_empty {
            self.done = true;
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.done {
            None
        } else {
            Some(&self.s)
        }
    }
}