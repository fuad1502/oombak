use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, RwLock, RwLockReadGuard},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, Frame};

use crate::{
    component::{Component, HandleResult},
    components::models::{PlotType, SimulationSpec},
    threads::RendererMessage,
    utils::bitvec_str::Radix,
    widgets::{CommandLineState, DropDownState, Form, FormState, InputField, KeyMaps},
};

pub struct SignalDisplayPropertiesSetter {
    simulation_spec: Arc<RwLock<SimulationSpec>>,
    signal_name: String,
    form_state: FormState,
    renderer_channel: Sender<RendererMessage>,
}

impl SignalDisplayPropertiesSetter {
    pub fn new(
        signal_name: String,
        simulation_spec: Arc<RwLock<SimulationSpec>>,
        renderer_channel: Sender<RendererMessage>,
    ) -> Self {
        let input_fields = vec![
            InputField::dropdown("Radix", &["Binary", "Hexadecimal", "Octal", "Decimal"]),
            InputField::dropdown("Signedness", &["Unsigned", "Signed"]),
            InputField::dropdown("Plot type", &["Digital", "Analog"]),
            InputField::text("Plot height"),
        ];
        let mut form_state = FormState::new(input_fields);
        Self::set_form_state_initial_values(
            &mut form_state,
            &signal_name,
            simulation_spec.read().unwrap(),
        );
        Self {
            simulation_spec,
            signal_name,
            form_state,
            renderer_channel,
        }
    }

    fn set_form_state_initial_values(
        form_state: &mut FormState,
        signal_name: &str,
        simulation_spec: RwLockReadGuard<'_, SimulationSpec>,
    ) {
        let state: &mut CommandLineState = form_state
            .get_input_state_mut(3)
            .unwrap()
            .try_into()
            .unwrap();
        let height = simulation_spec.get_wave_spec(signal_name).unwrap().height;
        state.set_text(&height.to_string());

        let state: &mut DropDownState = form_state
            .get_input_state_mut(2)
            .unwrap()
            .try_into()
            .unwrap();
        let plot_type = simulation_spec
            .get_wave_spec(signal_name)
            .unwrap()
            .plot_type;
        match plot_type {
            PlotType::Digital => state.select(0).unwrap(),
            PlotType::Analog => state.select(1).unwrap(),
        };

        let state: &mut DropDownState = form_state
            .get_input_state_mut(1)
            .unwrap()
            .try_into()
            .unwrap();
        let signed = simulation_spec.get_wave_spec(signal_name).unwrap().signed;
        if signed {
            state.select(1).unwrap()
        } else {
            state.select(0).unwrap()
        };

        let state: &mut DropDownState = form_state
            .get_input_state_mut(0)
            .unwrap()
            .try_into()
            .unwrap();
        let radix = simulation_spec.get_wave_spec(signal_name).unwrap().radix;
        match radix {
            Radix::Binary => state.select(0).unwrap(),
            Radix::Hexadecimal => state.select(1).unwrap(),
            Radix::Octal => state.select(2).unwrap(),
            Radix::Decimal => state.select(3).unwrap(),
        };
    }

    fn parse_user_input(entries: &[String]) -> Result<(Radix, bool, PlotType, u16), String> {
        let radix = match &entries[0][..] {
            "Binary" => Radix::Binary,
            "Hexadecimal" => Radix::Hexadecimal,
            "Octal" => Radix::Octal,
            "Decimal" => Radix::Decimal,
            _ => panic!(""),
        };

        let signed = match &entries[1][..] {
            "Unsigned" => false,
            "Signed" => true,
            _ => panic!(""),
        };

        let plot_type = match &entries[2][..] {
            "Analog" => PlotType::Analog,
            "Digital" => PlotType::Digital,
            _ => panic!(""),
        };

        let height = str::parse(&entries[3])
            .map_err(|e| format!("Cannot convert height input ({}) to u16: {}", entries[2], e))?;

        Ok((radix, signed, plot_type, height))
    }

    fn set_wave_spec_radix(&mut self, radix: Radix) {
        self.simulation_spec
            .write()
            .unwrap()
            .get_wave_spec_mut(&self.signal_name)
            .unwrap()
            .radix = radix;
    }

    fn set_wave_spec_signed(&mut self, signed: bool) {
        self.simulation_spec
            .write()
            .unwrap()
            .get_wave_spec_mut(&self.signal_name)
            .unwrap()
            .signed = signed;
    }

    fn set_wave_spec_plot_type(&mut self, plot_type: PlotType) {
        self.simulation_spec
            .write()
            .unwrap()
            .get_wave_spec_mut(&self.signal_name)
            .unwrap()
            .plot_type = plot_type;
    }

    fn set_wave_spec_height(&mut self, height: u16) {
        self.simulation_spec
            .write()
            .unwrap()
            .get_wave_spec_mut(&self.signal_name)
            .unwrap()
            .height = height;
    }
}

impl Component for SignalDisplayPropertiesSetter {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let form = Form::default();
        f.render_stateful_widget(form, rect, &mut self.form_state);
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult {
        match key_event.code {
            KeyCode::Char(ch) if self.form_state.is_command_line() => self.form_state.put(ch),
            KeyCode::Up | KeyCode::Char('k') => self.form_state.up(),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => self.form_state.down(),
            KeyCode::Left | KeyCode::Char('h') => self.form_state.left(),
            KeyCode::Right | KeyCode::Char('l') => self.form_state.right(),
            KeyCode::Backspace => self.form_state.backspace(),
            KeyCode::Enter => {
                if self.form_state.is_apply() {
                    let (radix, signed, plot_type, height) =
                        Self::parse_user_input(&self.form_state.entries()).unwrap();
                    self.set_wave_spec_radix(radix);
                    self.set_wave_spec_signed(signed);
                    self.set_wave_spec_plot_type(plot_type);
                    if matches!(plot_type, PlotType::Analog) {
                        self.set_wave_spec_height(height);
                    }
                    return HandleResult::ReleaseFocus;
                } else if self.form_state.is_cancel() {
                    return HandleResult::ReleaseFocus;
                } else if self.form_state.is_dropdown() {
                    self.form_state.enter();
                } else {
                    self.form_state.down();
                }
            }
            KeyCode::Char('q') => return HandleResult::ReleaseFocus,
            _ => (),
        }
        self.renderer_channel.send(RendererMessage::Render).unwrap();
        HandleResult::Handled
    }

    fn handle_resize_event(&mut self, _: u16, _: u16) -> HandleResult {
        HandleResult::NotHandled
    }

    fn handle_focus_gained(&mut self) -> HandleResult {
        unimplemented!()
    }

    fn get_focused_child(&self) -> Option<Arc<RwLock<dyn Component>>> {
        None
    }

    fn get_key_mappings(&self) -> KeyMaps {
        KeyMaps::from(HashMap::new())
    }
}
