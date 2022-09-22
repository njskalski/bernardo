use bernardo::widget::widget::WID;
use bernardo::widgets::dump_visualizer_widget::DumpVisualizerWidget;
use bernardo::widgets::with_scroll::WithScroll;

pub struct ReaderMainWidget {
    wid: WID,
    main_display: WithScroll<DumpVisualizerWidget>,
}

impl ReaderMainWidget {}