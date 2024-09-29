// The idea of this struct is to move common code from testbed builders to a single place
// so they can be re-used later.
// One should use it this way:
//
// pub type SomeWidgetTestbedBuilder = GenericWidgetTestbedBuilder<SomeWidgetTestbed>;
//
// impl SomeWidgetTestbedBuilder {
//  fn build() -> SomeWidgetTestbed {
//   SomeWidget {
//   }
//  }
// }

// pub struct GenericWidgetTestbedBuilder<W: Widget> {
// pub widget: W,
// pub size: XY,
// pub providers: Providers,
// pub last_frame: Option<MetaOutputFrame>,
// pub mock_navcomp_pilot: MockNavCompProviderPilot,
//
// pub output: MockOutput,
// pub recv: Receiver<MetaOutputFrame>,
// pub last_msg: Option<Box<dyn AnyMsg>>,
// }

use std::default::Default;
use std::marker::PhantomData;

use crate::config::config::Config;
use crate::mocks::mock_navcomp_provider::MockNavCompProvider;
use crate::mocks::mock_providers_builder::MockProvidersBuilder;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct GenericWidgetTestbedBuilder<W: Widget, AdditionalData> {
    pub size: Option<XY>,
    pub providers: MockProvidersBuilder,
    pub mock_nav_comp_provider: Option<MockNavCompProvider>,
    pub additional_data: AdditionalData,
    pub config: Option<Config>,
    pub step_frame: bool,
    pub _phantom_data: PhantomData<W>,
}

impl<W: Widget, AdditionalData> GenericWidgetTestbedBuilder<W, AdditionalData> {
    pub fn new(additional_data: AdditionalData) -> Self {
        GenericWidgetTestbedBuilder {
            size: None,
            providers: MockProvidersBuilder::default(),
            mock_nav_comp_provider: None,
            additional_data,
            config: None,
            step_frame: false,
            _phantom_data: Default::default(),
        }
    }

    pub fn with_size(mut self, size: XY) -> Self {
        self.size = Some(size);
        self
    }

    pub fn providers(&mut self) -> &mut MockProvidersBuilder {
        &mut self.providers
    }

    pub fn with_step_frame(mut self, step_frame: bool) -> Self {
        self.step_frame = step_frame;
        self
    }

    pub fn with_mock_nav_comp_provider(mut self, mock_nav_comp_provider: MockNavCompProvider) -> Self {
        self.mock_nav_comp_provider = Some(mock_nav_comp_provider);
        self
    }
}

impl<W: Widget, AdditionalData: Default> Default for GenericWidgetTestbedBuilder<W, AdditionalData> {
    fn default() -> Self {
        GenericWidgetTestbedBuilder {
            size: None,
            providers: MockProvidersBuilder::default(),
            mock_nav_comp_provider: None,
            additional_data: Default::default(),
            config: None,
            step_frame: false,
            _phantom_data: Default::default(),
        }
    }
}
