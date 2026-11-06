async function load_wasm() {
  const error_div = document.getElementById("error");
  try {
    const { default: init, SAP1Emulator } = await import(document.getElementById("script-data").dataset.loader);
    await init();
    
    const emulator = new SAP1Emulator({
      memory_cell_id: [
        "memory_cell_0",
        "memory_cell_1",
        "memory_cell_2",
        "memory_cell_3",
        "memory_cell_4",
        "memory_cell_5",
        "memory_cell_6",
        "memory_cell_7",
        "memory_cell_8",
        "memory_cell_9",
        "memory_cell_A",
        "memory_cell_B",
        "memory_cell_C",
        "memory_cell_D",
        "memory_cell_E",
        "memory_cell_F"
      ],
      instruction_reg_id: "instruction_reg",
      memory_addr_reg_id: "memory_addr_reg",
      register_a_id: "register_a",
      register_b_id: "register_b",
      out_register_id: "out_register",
      pc_register_id: "pc_register",
      is_halted_id: "is_halted",
      stage_id: "stage",
      decoded_opcode_id: "decoded_opcode",
      decoded_operand_id: "decoded_operand",
      bus_id: "bus",
      alu_output_id: "alu_output",
      alu_opcode_id: "alu_opcode",
      status_id: "status",
      initial_memory: [
        0x0C,
        0x2B,
        0x1D,
        0x1A,
        0x2E,
        0x29,
        0xE0,
        0xF0,
        0x00,
        0x01,
        0x03,
        0x02,
        0x04,
        0x05,
        0x06,
        0x00
      ],
      description_id: "description",
    });
    
    function doStepping() {
      try {
        emulator.advance()
        error_div.hidden = true
        return true
      } catch (error) {
        error_div.textContent = "From emulator: " + error;
        error_div.hidden = false
        return false
      }
    }
    
    const step_button = document.getElementById("step")
    step_button.onclick = () => {
      doStepping()
    }
    
    const reset_button = document.getElementById("reset")
    reset_button.onclick = () => {
      try {
        emulator.reset()
        error_div.hidden = true
      } catch (error) {
        error_div.textContent = "From emulator: " + error;
        error_div.hidden = false
      }
    }
    
    const reset_memory_button = document.getElementById("reset_memory")
    reset_memory_button.onclick = () => {
      emulator.reset_memory()
    }
    
    let is_auto_run = false
    let autorun_intervalid;
    
    const run_button = document.getElementById("run")
    
    function stopAutoRun() {
      step_button.disabled = false
      reset_button.disabled = false
      reset_memory_button.disabled = false
      run_button.textContent = "Run"
      is_auto_run = false
      clearInterval(autorun_intervalid)
    }
    
    run_button.onclick = () => {
      if (is_auto_run) {
        stopAutoRun()
      } else {
        step_button.disabled = true
        reset_button.disabled = true
        reset_memory_button.disabled = true
        run_button.textContent = "Stop"
        is_auto_run = true;
        
        autorun_intervalid = setInterval(() => {
          if (!doStepping() || emulator.is_halted()) {
            stopAutoRun()
          }
        }, 500)
      }
    }
  } catch (error) {
    error_div.textContent = "Error initializing emulator: " + error;
    error_div.hidden = false
  }
}

load_wasm()