AC-9 — WriteToolInvalidationMap Runtime Extensibility (TD-S-PLUGIN-PREREQ-A-003 Closed)
========================================================================================
Story: S-PLUGIN-PREREQ-E (v1.50) | BC: BC-2.16.012 + BC-2.16.002 | HEAD: 051eab95

EVIDENCE TYPE: Test output (3 unit tests + VP-156 proptest suite) + boot-test output +
               source excerpt (register_write_tool callsite)

-------------------------------------------------------------------------------
SOURCE EXCERPT: crates/prism-query/src/invalidation.rs — key APIs
-------------------------------------------------------------------------------

  static QUERY_PHASE_STARTED: AtomicBool = AtomicBool::new(false);  (line 95)

  pub fn mark_query_phase_started() {
      QUERY_PHASE_STARTED.store(true, Ordering::Release);            (line 110-111)
  }

  pub fn register_write_tool(entry: WriteToolInvalidationMap) -> Result<(), SpecEngineError> {
      if QUERY_PHASE_STARTED.load(Ordering::Acquire) {
          tracing::warn!(
              event_type = "write_tool_registration_after_boot",
              plugin_name = %entry.plugin_name,
              tool_name = %entry.tool_name,
              error = "E-PLUGIN-020"                                 (lines 124-133)
          );
          return Err(SpecEngineError::WriteToolRegistrationAfterBoot);
      }
      ...
  }

  WriteToolInvalidationMap struct includes `plugin_name: String` field (line 79)
  set by PluginRuntime from plugin manifest `name` field per ADR-026 D7 v1.23.

-------------------------------------------------------------------------------
TEST OUTPUT: Three unit tests (happy-path + duplicate + post-boot)
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-query -E 'test(BC_2_16_012)' --no-fail-fast

    Starting 3 tests across 12 binaries (906 tests skipped)
        PASS [   0.029s] (1/3) prism-query invalidation::tests::test_BC_2_16_012_003_write_tool_invalidation_runtime_register_happy_path
        PASS [   0.030s] (2/3) prism-query invalidation::tests::test_BC_2_16_012_003_write_tool_invalidation_duplicate_rejected
        PASS [   0.030s] (3/3) prism-query invalidation::tests::test_BC_2_16_012_003_write_tool_invalidation_post_boot_rejected_with_warn_event
    Summary [   0.031s] 3 tests run: 3 passed, 906 skipped

  Test 1 (happy-path): register_write_tool(entry).is_ok() = true;
    entry is visible on next read-guard acquisition. PASS.

  Test 2 (duplicate): register_write_tool with same tool_name returns Err(E-PLUGIN-012).
    PASS.

  Test 3 (post-boot): Calls mark_query_phase_started() (public API, as boot.rs will call it),
    then register_write_tool(entry). Asserts Err(WriteToolRegistrationAfterBoot) (E-PLUGIN-020).
    Captures WARN tracing event (tracing-test = "0.2" subscriber) with fields:
      event_type = "write_tool_registration_after_boot"
      plugin_name = <plugin>
      tool_name = <tool>
      error = "E-PLUGIN-020"
    All fields verified. PASS.

-------------------------------------------------------------------------------
TEST OUTPUT: VP-156 proptest suite (5 property tests)
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-query --test vp156_write_tool_registration_uniqueness
         --test vp156_write_tool_post_boot_proptest --features test-helpers --no-fail-fast

    Starting 5 tests across 2 binaries
        PASS [   0.072s] (1/5) prism-query::vp156_write_tool_registration_uniqueness prop_register_write_tool_idempotency_under_dedup_key
        PASS [   0.073s] (2/5) prism-query::vp156_write_tool_registration_uniqueness prop_register_write_tool_first_call_succeeds
        PASS [   0.075s] (3/5) prism-query::vp156_write_tool_post_boot_proptest prop_register_write_tool_post_boot_rejected_with_e_plugin_020
        PASS [   0.104s] (4/5) prism-query::vp156_write_tool_registration_uniqueness prop_register_write_tool_duplicate_rejected_with_e_plugin_012
        PASS [   0.108s] (5/5) prism-query::vp156_write_tool_registration_uniqueness prop_register_write_tool_distinct_keys_accepted
    Summary [   0.108s] 5 tests run: 5 passed, 0 skipped

-------------------------------------------------------------------------------
TEST OUTPUT: prism-bin plugin_boot_tests (15 tests; AC-9 write-tool coverage)
-------------------------------------------------------------------------------

Command: cargo nextest run -p prism-bin --test plugin_boot_tests --no-fail-fast

    Starting 15 tests across 1 binary
        ...
        PASS [   0.097s] ( 7/15) prism-bin::plugin_boot_tests test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin
        ...
        PASS [   0.105s] (10/15) prism-bin::plugin_boot_tests test_BC_2_16_012_plugin_runtime_registers_write_tools_pre_query_phase
        ...
        PASS [   0.111s] (14/15) prism-bin::plugin_boot_tests test_BC_2_16_012_write_tool_reg_failure_rolls_back_plugin
    Summary [   0.112s] 15 tests run: 15 passed, 0 skipped

  test_BC_2_16_012_plugin_runtime_registers_write_tools_pre_query_phase:
    Verifies PluginRuntime calls register_write_tool before mark_query_phase_started()
    (step 7.5 must complete before step 8 sets the flag). PASS.

  test_BC_2_16_012_write_tool_reg_failure_rolls_back_plugin (F-P4-002 fail-closed):
    Verifies that a failed register_write_tool causes plugin rollback. PASS.

  test_BC_2_16_012_write_tool_reg_failure_rolls_back_all_remaining_tools_for_plugin (F-P6-001 3-tool atomic rollback):
    Verifies that on a second-tool registration failure, the first tool is also rolled back
    (atomic rollback eliminates orphaned-tool class). PASS.

TD-S-PLUGIN-PREREQ-A-003 CLOSURE CONFIRMED:
  WriteToolInvalidationMap container is RwLock<Vec<WriteToolInvalidationMap>>.
  register_write_tool() API exists and is callable after startup.
  AtomicBool QUERY_PHASE_STARTED flag set by mark_query_phase_started().
  Tracing event BC-2.16.002 row 33 field schema verified by test_BC_2_16_012_003_..._post_boot.

RESULT: PASS — All 3 unit tests + 5 VP-156 proptests + 3 relevant boot tests = 11 tests
total. TD-S-PLUGIN-PREREQ-A-003 closed. INV-INVALIDATION-EXT-001 holds.
