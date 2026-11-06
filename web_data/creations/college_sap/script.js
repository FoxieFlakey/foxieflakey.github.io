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
      active_read_led_class: "read_led_active",
      active_write_led_class: "write_led_active",
      pc_register_status: {
        read_led_id: "pc_register_read_led",
        write_led_id: "pc_register_write_led",
        status_id: "pc_register_status_text",
      },
      memory_addr_reg_status: {
        read_led_id: "memory_addr_reg_read_led",
        write_led_id: "memory_addr_reg_write_led",
        status_id: "memory_addr_reg_status_text",
      },
      decoded_operand_status: {
        read_led_id: "decoded_operand_read_led",
        write_led_id: "decoded_operand_write_led",
        status_id: "decoded_operand_status_text",
      },
      decoded_opcode_status: {
        read_led_id: "decoded_opcode_read_led",
        write_led_id: "decoded_opcode_write_led",
        status_id: "decoded_opcode_status_text",
      },
      register_a_status: {
        read_led_id: "register_a_read_led",
        write_led_id: "register_a_write_led",
        status_id: "register_a_status_text",
      },
      register_b_status: {
        read_led_id: "register_b_read_led",
        write_led_id: "register_b_write_led",
        status_id: "register_b_status_text",
      },
      output_register_status: {
        read_led_id: "out_register_read_led",
        write_led_id: "out_register_write_led",
        status_id: "out_register_status_text",
      },
      instruction_register_status: {
        read_led_id: "instruction_reg_read_led",
        write_led_id: "instruction_reg_write_led",
        status_id: "instruction_reg_status_text",
      },
      alu_output_status: {
        read_led_id: "alu_output_read_led",
        write_led_id: "alu_output_write_led",
        status_id: "alu_output_status_text",
      },
      memory_cell_status: [
        {
          read_led_id: "memory_cell_0_read_led",
          write_led_id: "memory_cell_0_write_led",
          status_id: "memory_cell_0_status_text",
        },
        {
          read_led_id: "memory_cell_1_read_led",
          write_led_id: "memory_cell_1_write_led",
          status_id: "memory_cell_1_status_text",
        },
        {
          read_led_id: "memory_cell_2_read_led",
          write_led_id: "memory_cell_2_write_led",
          status_id: "memory_cell_2_status_text",
        },
        {
          read_led_id: "memory_cell_3_read_led",
          write_led_id: "memory_cell_3_write_led",
          status_id: "memory_cell_3_status_text",
        },
        {
          read_led_id: "memory_cell_4_read_led",
          write_led_id: "memory_cell_4_write_led",
          status_id: "memory_cell_4_status_text",
        },
        {
          read_led_id: "memory_cell_5_read_led",
          write_led_id: "memory_cell_5_write_led",
          status_id: "memory_cell_5_status_text",
        },
        {
          read_led_id: "memory_cell_6_read_led",
          write_led_id: "memory_cell_6_write_led",
          status_id: "memory_cell_6_status_text",
        },
        {
          read_led_id: "memory_cell_7_read_led",
          write_led_id: "memory_cell_7_write_led",
          status_id: "memory_cell_7_status_text",
        },
        {
          read_led_id: "memory_cell_8_read_led",
          write_led_id: "memory_cell_8_write_led",
          status_id: "memory_cell_8_status_text",
        },
        {
          read_led_id: "memory_cell_9_read_led",
          write_led_id: "memory_cell_9_write_led",
          status_id: "memory_cell_9_status_text",
        },
        {
          read_led_id: "memory_cell_A_read_led",
          write_led_id: "memory_cell_A_write_led",
          status_id: "memory_cell_A_status_text",
        },
        {
          read_led_id: "memory_cell_B_read_led",
          write_led_id: "memory_cell_B_write_led",
          status_id: "memory_cell_B_status_text",
        },
        {
          read_led_id: "memory_cell_C_read_led",
          write_led_id: "memory_cell_C_write_led",
          status_id: "memory_cell_C_status_text",
        },
        {
          read_led_id: "memory_cell_D_read_led",
          write_led_id: "memory_cell_D_write_led",
          status_id: "memory_cell_D_status_text",
        },
        {
          read_led_id: "memory_cell_E_read_led",
          write_led_id: "memory_cell_E_write_led",
          status_id: "memory_cell_E_status_text",
        },
        {
          read_led_id: "memory_cell_F_read_led",
          write_led_id: "memory_cell_F_write_led",
          status_id: "memory_cell_F_status_text",
        },
      ],
      mem_status_line_id: [
        "status_line_memory_cell_0",
        "status_line_memory_cell_1",
        "status_line_memory_cell_2",
        "status_line_memory_cell_3",
        "status_line_memory_cell_4",
        "status_line_memory_cell_5",
        "status_line_memory_cell_6",
        "status_line_memory_cell_7",
        "status_line_memory_cell_8",
        "status_line_memory_cell_9",
        "status_line_memory_cell_A",
        "status_line_memory_cell_B",
        "status_line_memory_cell_C",
        "status_line_memory_cell_D",
        "status_line_memory_cell_E",
        "status_line_memory_cell_F",
      ],
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