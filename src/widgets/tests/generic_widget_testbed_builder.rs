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

use crate::gladius::providers::Providers;
use crate::mocks::mock_navcomp_provider::MockNavCompProvider;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

pub struct GenericWidgetTestbedBuilder<W: Widget, AdditionalData> {
    pub size: Option<XY>,
    pub providers: Option<Providers>,
    pub mock_nav_comp_provider: Option<MockNavCompProvider>,
    pub additional_data: AdditionalData,
    pub _phantom_data: PhantomData<W>,
}

impl<W: Widget, AdditionalData> GenericWidgetTestbedBuilder<W, AdditionalData> {
    pub fn new(additional_data: AdditionalData) -> Self {
        GenericWidgetTestbedBuilder {
            size: None,
            providers: None,
            mock_nav_comp_provider: None,
            additional_data,
            _phantom_data: Default::default(),
        }
    }

    pub fn with_size(self, size: XY) -> Self {
        Self {
            size: Some(size),
            ..self
        }
    }

    pub fn with_provider(self, provider: Providers) -> Self {
        Self {
            providers: Some(provider),
            ..self
        }
    }

    pub fn with_mock_nav_comp_provider(self, mock_nav_comp_provider: MockNavCompProvider) -> Self {
        Self {
            mock_nav_comp_provider: Some(mock_nav_comp_provider),
            ..self
        }
    }
}

impl<W: Widget, AdditionalData: Default> Default for GenericWidgetTestbedBuilder<W, AdditionalData> {
    fn default() -> Self {
        GenericWidgetTestbedBuilder {
            size: None,
            providers: None,
            mock_nav_comp_provider: None,
            additional_data: Default::default(),
            _phantom_data: Default::default(),
        }
    }
}