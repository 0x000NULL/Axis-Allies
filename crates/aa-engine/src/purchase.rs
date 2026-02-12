//! Purchase & Repair phase helpers and tests.

#[cfg(test)]
mod tests {
    use crate::action::Action;
    use crate::error::EngineError;
    use crate::phase::{Phase, PhaseState, PurchaseState};
    use crate::power::Power;
    use crate::territory::{Facility, FacilityType, TerritoryState};
    use crate::unit::UnitType;
    use crate::Engine;

    /// Create an engine and return it in the PurchaseAndRepair phase for Germany.
    fn setup_engine() -> Engine {
        Engine::new_game(42)
    }

    // ---- PurchaseUnit tests ----

    #[test]
    fn test_purchase_infantry() {
        let mut engine = setup_engine();
        let ipcs_before = engine.state().powers[Power::Germany as usize].ipcs;

        let result = engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 2,
            })
            .unwrap();

        assert_eq!(result.events.len(), 1);
        let ipcs_after = engine.state().powers[Power::Germany as usize].ipcs;
        assert_eq!(ipcs_after, ipcs_before - 6); // 2 * 3 IPC

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert_eq!(ps.purchases.len(), 1);
            assert_eq!(ps.purchases[0], (UnitType::Infantry, 2));
            assert_eq!(ps.ipcs_spent, 6);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_purchase_multiple_types() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();
        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Tank,
                count: 1,
            })
            .unwrap();

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert_eq!(ps.purchases.len(), 2);
            assert_eq!(ps.ipcs_spent, 15); // 3*3 + 1*6
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_purchase_same_type_accumulates() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 2,
            })
            .unwrap();
        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert_eq!(ps.purchases.len(), 1);
            assert_eq!(ps.purchases[0], (UnitType::Infantry, 5));
            assert_eq!(ps.ipcs_spent, 15);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_purchase_insufficient_ipcs() {
        let mut engine = setup_engine();
        // Germany starts with 30 IPCs, battleship costs 20 each
        let result = engine.submit_action(Action::PurchaseUnit {
            unit_type: UnitType::Battleship,
            count: 2,
        });
        assert!(matches!(
            result,
            Err(EngineError::InsufficientIPCs { needed: 40, available: 30 })
        ));
    }

    #[test]
    fn test_purchase_zero_count_rejected() {
        let mut engine = setup_engine();
        let result = engine.submit_action(Action::PurchaseUnit {
            unit_type: UnitType::Infantry,
            count: 0,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    #[test]
    fn test_purchase_wrong_phase() {
        let mut engine = setup_engine();
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        // Now in CombatMovement
        let result = engine.submit_action(Action::PurchaseUnit {
            unit_type: UnitType::Infantry,
            count: 1,
        });
        assert!(matches!(result, Err(EngineError::WrongPhase { .. })));
    }

    // ---- China restriction ----

    #[test]
    fn test_china_can_only_buy_infantry() {
        let mut engine = setup_engine();
        // Advance to China's turn
        for _ in 0..4 {
            // Germany, Soviet, Japan, US
            engine.submit_action(Action::ConfirmPurchases).unwrap();
            engine.submit_action(Action::ConfirmCombatMovement).unwrap();
            engine.submit_action(Action::ConfirmPhase).unwrap();
            engine.submit_action(Action::ConfirmNonCombatMovement).unwrap();
            engine.submit_action(Action::ConfirmMobilization).unwrap();
            engine.submit_action(Action::ConfirmIncome).unwrap();
        }
        assert_eq!(engine.state().current_power, Power::China);

        // Infantry should work
        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 1,
            })
            .unwrap();

        // Tank should fail
        let result = engine.submit_action(Action::PurchaseUnit {
            unit_type: UnitType::Tank,
            count: 1,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    // ---- RemovePurchase tests ----

    #[test]
    fn test_remove_purchase() {
        let mut engine = setup_engine();
        let ipcs_before = engine.state().powers[Power::Germany as usize].ipcs;

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();

        engine
            .submit_action(Action::RemovePurchase {
                unit_type: UnitType::Infantry,
                count: 1,
            })
            .unwrap();

        let ipcs_after = engine.state().powers[Power::Germany as usize].ipcs;
        assert_eq!(ipcs_after, ipcs_before - 6); // 2 remaining * 3 IPC

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert_eq!(ps.purchases[0], (UnitType::Infantry, 2));
            assert_eq!(ps.ipcs_spent, 6);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_remove_all_of_type() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();

        engine
            .submit_action(Action::RemovePurchase {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert!(ps.purchases.is_empty());
            assert_eq!(ps.ipcs_spent, 0);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_remove_more_than_purchased_fails() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 2,
            })
            .unwrap();

        let result = engine.submit_action(Action::RemovePurchase {
            unit_type: UnitType::Infantry,
            count: 5,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    #[test]
    fn test_remove_unpurchased_type_fails() {
        let mut engine = setup_engine();
        let result = engine.submit_action(Action::RemovePurchase {
            unit_type: UnitType::Tank,
            count: 1,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    // ---- Undo purchase tests ----

    #[test]
    fn test_undo_purchase() {
        let mut engine = setup_engine();
        let ipcs_before = engine.state().powers[Power::Germany as usize].ipcs;

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Tank,
                count: 2,
            })
            .unwrap();

        assert!(engine.can_undo());
        engine.submit_action(Action::Undo).unwrap();

        let ipcs_after = engine.state().powers[Power::Germany as usize].ipcs;
        assert_eq!(ipcs_after, ipcs_before);

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert!(ps.purchases.is_empty());
            assert_eq!(ps.ipcs_spent, 0);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    #[test]
    fn test_undo_remove_purchase() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 3,
            })
            .unwrap();

        engine
            .submit_action(Action::RemovePurchase {
                unit_type: UnitType::Infantry,
                count: 1,
            })
            .unwrap();

        // Undo the remove — should restore the 3rd infantry
        engine.submit_action(Action::Undo).unwrap();

        if let PhaseState::Purchase(ref ps) = engine.state().phase_state {
            assert_eq!(ps.purchases[0], (UnitType::Infantry, 3));
            assert_eq!(ps.ipcs_spent, 9);
        } else {
            panic!("Expected Purchase phase state");
        }
    }

    // ---- ConfirmPurchases tests ----

    #[test]
    fn test_confirm_purchases_transitions_phase() {
        let mut engine = setup_engine();

        engine
            .submit_action(Action::PurchaseUnit {
                unit_type: UnitType::Infantry,
                count: 2,
            })
            .unwrap();

        let result = engine.submit_action(Action::ConfirmPurchases).unwrap();
        assert_eq!(engine.state().current_phase, Phase::CombatMovement);
        assert!(result.events.iter().any(|e| matches!(e, crate::action::GameEvent::PhaseChanged { .. })));
    }

    #[test]
    fn test_confirm_purchases_with_no_purchases() {
        let mut engine = setup_engine();
        // Should be valid to confirm with nothing purchased
        engine.submit_action(Action::ConfirmPurchases).unwrap();
        assert_eq!(engine.state().current_phase, Phase::CombatMovement);
    }

    // ---- RepairFacility tests ----

    #[test]
    fn test_repair_facility() {
        let mut engine = setup_engine();

        // Manually add a damaged factory to a German territory
        let tid = 0; // First territory
        engine.state_mut().territories[tid].owner = Some(Power::Germany);
        engine.state_mut().territories[tid].facilities.push(Facility {
            facility_type: FacilityType::MajorIndustrialComplex,
            damage: 5,
            max_damage: 20,
            operational: true,
        });

        let ipcs_before = engine.state().powers[Power::Germany as usize].ipcs;

        engine
            .submit_action(Action::RepairFacility {
                territory_id: tid as u16,
                damage_to_repair: 3,
            })
            .unwrap();

        let ipcs_after = engine.state().powers[Power::Germany as usize].ipcs;
        assert_eq!(ipcs_after, ipcs_before - 3);

        assert_eq!(engine.state().territories[tid].facilities[0].damage, 2);
    }

    #[test]
    fn test_repair_exceeds_damage_fails() {
        let mut engine = setup_engine();

        let tid = 0;
        engine.state_mut().territories[tid].owner = Some(Power::Germany);
        engine.state_mut().territories[tid].facilities.push(Facility {
            facility_type: FacilityType::MajorIndustrialComplex,
            damage: 3,
            max_damage: 20,
            operational: true,
        });

        let result = engine.submit_action(Action::RepairFacility {
            territory_id: tid as u16,
            damage_to_repair: 5,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    #[test]
    fn test_repair_no_facility_fails() {
        let mut engine = setup_engine();

        let tid = 0;
        engine.state_mut().territories[tid].owner = Some(Power::Germany);
        // No facilities

        let result = engine.submit_action(Action::RepairFacility {
            territory_id: tid as u16,
            damage_to_repair: 1,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    #[test]
    fn test_repair_enemy_territory_fails() {
        let mut engine = setup_engine();

        let tid = 0;
        engine.state_mut().territories[tid].owner = Some(Power::UnitedKingdom);
        engine.state_mut().territories[tid].facilities.push(Facility {
            facility_type: FacilityType::MajorIndustrialComplex,
            damage: 3,
            max_damage: 20,
            operational: true,
        });

        let result = engine.submit_action(Action::RepairFacility {
            territory_id: tid as u16,
            damage_to_repair: 1,
        });
        assert!(matches!(result, Err(EngineError::InvalidAction { .. })));
    }

    #[test]
    fn test_repair_insufficient_ipcs() {
        let mut engine = setup_engine();

        let tid = 0;
        engine.state_mut().territories[tid].owner = Some(Power::Germany);
        engine.state_mut().territories[tid].facilities.push(Facility {
            facility_type: FacilityType::MajorIndustrialComplex,
            damage: 50,
            max_damage: 100,
            operational: true,
        });

        // Germany has 30 IPCs
        let result = engine.submit_action(Action::RepairFacility {
            territory_id: tid as u16,
            damage_to_repair: 31,
        });
        assert!(matches!(
            result,
            Err(EngineError::InsufficientIPCs { .. })
        ));
    }
}
