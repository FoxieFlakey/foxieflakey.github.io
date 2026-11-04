use libcollege_saps::sap1::{ExecutionError, SAP1};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlInputElement, window};

#[wasm_bindgen]
pub struct SAP1Emulator {
    emulator: SAP1,
    memory_cells_elems: [HtmlInputElement; 16],
    instruction_reg_element: Element,
    memory_addr_reg_element: Element,
    register_a_element: Element,
    register_b_element: Element,
    out_register_element: Element,
    pc_register_element: Element,
    is_halted_element: Element,
    stage_element: Element,
    decoded_opcode_element: Element,
    decoded_operand_element: Element,
    bus_element: Element,
    alu_output_element: Element,
    alu_opcode_element: Element,
    status_element: Element,
}

#[derive(Deserialize, Serialize)]
pub struct SAP1Config {
    pub memory_cell_id: [String; 16],
    pub instruction_reg_id: String,
    pub memory_addr_reg_id: String,
    pub register_a_id: String,
    pub register_b_id: String,
    pub out_register_id: String,
    pub pc_register_id: String,
    pub is_halted_id: String,
    pub stage_id: String,
    pub decoded_opcode_id: String,
    pub decoded_operand_id: String,
    pub bus_id: String,
    pub alu_output_id: String,
    pub alu_opcode_id: String,
    pub status_id: String,
    pub initial_memory: Option<[u8; 16]>,
}

#[wasm_bindgen]
impl SAP1Emulator {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Self, String> {
        let config = serde_wasm_bindgen::from_value::<SAP1Config>(config)
            .map_err(|e| format!("Cannot deserialize config for SAP1 emulator: {e}"))?;
        let doc = window()
            .ok_or_else(|| "Cannot find 'window'")?
            .document()
            .ok_or_else(|| "Cannot find 'window.document'")?;

        let find_elem = |config_key: &str, id: &str| -> Result<Element, String> {
            doc.get_element_by_id(id).ok_or_else(|| {
                format!("Cannot find element '{id}' by id for '{config_key}' element")
            })
        };

        let mut cells = Vec::new();
        for (idx, id) in config.memory_cell_id.iter().enumerate() {
            let config_key = format!("memory_cell_id[{idx}]");
            let elem = find_elem(&config_key, id)?.dyn_into().map_err(|_| {
                format!(
                    "Element for memory cell at ID {id} for cell {config_key} is not input element!"
                )
            })?;
            cells.push(elem);
        }

        let mut this = Self {
            emulator: SAP1::new(),
            memory_cells_elems: cells
                .try_into()
                .expect("Apparently config.memory_cell_id is not 16"),
            instruction_reg_element: find_elem("instruction_reg", &config.instruction_reg_id)?,
            memory_addr_reg_element: find_elem("memory_addr_reg", &config.memory_addr_reg_id)?,
            register_a_element: find_elem("register_a", &config.register_a_id)?,
            register_b_element: find_elem("register_b", &config.register_b_id)?,
            out_register_element: find_elem("out_register", &config.out_register_id)?,
            pc_register_element: find_elem("pc_register", &config.pc_register_id)?,
            is_halted_element: find_elem("is_halted", &config.is_halted_id)?,
            stage_element: find_elem("stage", &config.stage_id)?,
            decoded_opcode_element: find_elem("decoded_opcode", &config.decoded_opcode_id)?,
            decoded_operand_element: find_elem("decoded_operand", &config.decoded_operand_id)?,
            bus_element: find_elem("bus", &config.bus_id)?,
            alu_output_element: find_elem("alu_output", &config.alu_output_id)?,
            alu_opcode_element: find_elem("alu_opcode", &config.alu_opcode_id)?,
            status_element: find_elem("status", &config.status_id)?,
        };

        if let Some(memory) = config.initial_memory {
            this.emulator.memory = memory;
        }
        this.refresh_display();
        Ok(this)
    }

    pub fn is_halted(&self) -> bool {
        self.emulator.is_halted
    }

    pub fn reset(&mut self) -> Result<(), String> {
        self.reread_inputs()
            .map_err(|e| format!("Cannot reset emulation: {e}"))?;

        let saved_mem = self.emulator.memory;
        self.emulator = SAP1::new();
        self.emulator.memory = saved_mem;
        self.refresh_display();
        Ok(())
    }

    pub fn reset_memory(&mut self) {
        self.emulator.memory = [0; 16];
        self.refresh_display();
    }

    pub fn advance(&mut self) -> Result<(), String> {
        self.reread_inputs()
            .map_err(|e| format!("Cannot advance emulation: {e}"))?;
        self.emulator.advance().map_err(|e| match e {
            ExecutionError::MachineIsHalted => format!("Machine is halted, cannot execute"),
            ExecutionError::UnknownOperation { opcode, operand } => {
                format!("Machine encountered unknown opcode: {opcode} with operand {operand}")
            }
        })?;

        self.refresh_display();
        Ok(())
    }
}

impl SAP1Emulator {
    fn reread_inputs(&mut self) -> Result<(), String> {
        for (idx, elem) in self.memory_cells_elems.iter().enumerate() {
            let cell_value = elem.value();
            let val;

            if let Some(hex_string) = cell_value.strip_prefix("0x") {
                val = u8::from_str_radix(hex_string, 16).map_err(|e| {
                    format!("Reading memory at 0x{idx:02x} got invalid hex number: {e}")
                })?;
            } else if let Some(binary_string) = cell_value.strip_prefix("0b") {
                val = u8::from_str_radix(binary_string, 2).map_err(|e| {
                    format!("Reading memory at 0x{idx:02x} got invalid binary number: {e}")
                })?;
            } else if let Some(octal_string) = cell_value.strip_prefix("0o") {
                val = u8::from_str_radix(octal_string, 8).map_err(|e| {
                    format!("Reading memory at 0x{idx:02x} got invalid octal number: {e}")
                })?;
            } else {
                val = cell_value.parse::<u8>().map_err(|e| {
                    format!("Reading memory at 0x{idx:02x} got invalid number: {e}")
                })?;
            }

            self.emulator.memory[idx] = val;
        }
        Ok(())
    }

    fn refresh_display(&self) {
        for (idx, elem) in self.memory_cells_elems.iter().enumerate() {
            elem.set_value(&format!("0x{:02x}", self.emulator.memory[idx]));
        }

        self.instruction_reg_element.set_text_content(Some(&format!(
            "0x{:02x}",
            self.emulator.instruction_register
        )));
        self.memory_addr_reg_element.set_text_content(Some(&format!(
            "0x{:02x}",
            self.emulator.memory_address_register
        )));
        self.register_a_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.accumulator)));
        self.register_b_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.register_b)));
        self.out_register_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.out_register)));
        self.pc_register_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.program_counter)));
        self.is_halted_element
            .set_text_content(Some(if self.emulator.is_halted {
                "true"
            } else {
                "false"
            }));
        self.stage_element
            .set_text_content(Some(match self.emulator.stage {
                libcollege_saps::sap1::Stage::Fetch => "Fetching",
                libcollege_saps::sap1::Stage::Decode => "Decoding",
                libcollege_saps::sap1::Stage::Execute1 => "Executing (Part 1)",
                libcollege_saps::sap1::Stage::Execute2 => "Executing (Part 2)",
                libcollege_saps::sap1::Stage::Execute3 => "Executing (Part 3)",
                libcollege_saps::sap1::Stage::Increment => "Incrementing PC",
            }));
        self.decoded_opcode_element
            .set_text_content(Some(match self.emulator.decoded_op {
                libcollege_saps::sap1::Opcode::LoadA => "LoadA",
                libcollege_saps::sap1::Opcode::Add => "Add",
                libcollege_saps::sap1::Opcode::Sub => "Sub",
                libcollege_saps::sap1::Opcode::Out => "Out",
                libcollege_saps::sap1::Opcode::Halt => "Halt",
            }));
        self.decoded_operand_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.decoded_operand)));
        self.bus_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.bus)));
        self.alu_output_element
            .set_text_content(Some(&format!("0x{:02x}", self.emulator.alu_output)));
        self.alu_opcode_element
            .set_text_content(Some(match self.emulator.alu_opcode {
                libcollege_saps::sap1::ALUOpcode::Add => "Addition",
                libcollege_saps::sap1::ALUOpcode::Sub => "Subtraction",
            }));

        self.status_element
            .set_text_content(Some(if self.is_halted() {
                "Halted"
            } else {
                "Not halted"
            }));
    }
}
