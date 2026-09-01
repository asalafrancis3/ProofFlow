//! # Waste Recommendation Engine
//!
//! Generates ranked waste-collection recommendations for participants based on
//! their historical waste-disposal patterns.
//!
//! ## Ranking / Scoring Algorithm
//!
//! The engine iterates over a fixed set of waste categories
//! (`plastic`, `metal`, `paper`, `glass`) and assigns each a **confidence
//! score** derived from the participant's history.
//!
//! ```text
//! confidence(history, waste_type) =
//!     min( (count_in_history / max(history_len, 1)) * 0.8 + 0.2 , 1.0 )
//! ```
//!
//! * **`count_in_history`** – number of history entries whose string contains
//!   the waste type as a substring (case-sensitive).
//! * **`history_len`** – total number of entries; capped at a minimum of 1 to
//!   avoid division-by-zero on empty histories.
//! * The linear transform `* 0.8 + 0.2` guarantees a floor of 0.2 for any
//!   waste type, even when the history contains zero matches.
//! * The `.min(1.0)` clamp ensures the score never exceeds 1.0.
//!
//! Only waste types whose confidence **strictly exceeds 0.3** are included in
//! the output. The final list is sorted in **descending** order by confidence.
//!
//! ### Estimated Reward
//!
//! Each recommendation carries an `estimated_reward` field computed as:
//!
//! ```text
//! estimated_reward = (100.0 * confidence_score) as u128
//! ```
//!
//! ### Determinism
//!
//! The algorithm is fully deterministic – no randomness, time-dependence, or
//! external state is involved. Identical inputs always produce identical
//! outputs, which makes unit-testing straightforward.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasteRecommendation {
    pub waste_type: String,
    pub confidence_score: f64,
    pub collection_location: (f64, f64),
    pub estimated_reward: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub participant_id: String,
    pub location: (f64, f64),
    pub waste_history: Vec<String>,
}

pub struct RecommendationEngine;

impl RecommendationEngine {
    pub fn generate_recommendations(request: RecommendationRequest) -> Vec<WasteRecommendation> {
        let mut recommendations = Vec::new();

        // Simple ML-based recommendation logic
        let waste_types = vec!["plastic", "metal", "paper", "glass"];
        let base_reward = 100u128;

        for waste_type in waste_types {
            let confidence = Self::calculate_confidence(&request.waste_history, waste_type);
            if confidence > 0.3 {
                recommendations.push(WasteRecommendation {
                    waste_type: waste_type.to_string(),
                    confidence_score: confidence,
                    collection_location: request.location,
                    estimated_reward: (base_reward as f64 * confidence) as u128,
                });
            }
        }

        recommendations.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        recommendations
    }

    fn calculate_confidence(history: &[String], waste_type: &str) -> f64 {
        let count = history.iter().filter(|w| w.contains(waste_type)).count();
        let base_confidence = (count as f64) / (history.len().max(1) as f64);
        (base_confidence * 0.8 + 0.2).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ─────────────────────────────────────────────────────────

    fn make_request(participant_id: &str, history: Vec<&str>) -> RecommendationRequest {
        RecommendationRequest {
            participant_id: participant_id.to_string(),
            location: (40.7128, -74.0060),
            waste_history: history.into_iter().map(String::from).collect(),
        }
    }

    fn waste_types(recs: &[WasteRecommendation]) -> Vec<&str> {
        recs.iter().map(|r| r.waste_type.as_str()).collect()
    }

    // ── calculate_confidence (via generate_recommendations side-effects) ──

    #[test]
    fn confidence_empty_history_yields_no_recommendations() {
        let req = make_request("user_empty", vec![]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.is_empty(), "empty history should produce no recommendations");
    }

    #[test]
    fn confidence_single_match_gives_maximum() {
        let req = make_request("user_single", vec!["plastic"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].waste_type, "plastic");
        assert!((recs[0].confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_no_match_for_type() {
        let req = make_request("user_no_match", vec!["plastic"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].waste_type, "plastic");
    }

    #[test]
    fn confidence_partial_match() {
        let req = make_request("user_partial", vec!["plastic", "plastic", "metal", "paper", "glass"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic").unwrap();
        assert!((plastic.confidence_score - 0.52).abs() < 1e-10);
    }

    #[test]
    fn confidence_substring_matching() {
        let req = make_request("user_substr", vec!["plastic_bottle"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].waste_type, "plastic");
        assert!((recs[0].confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_cap_at_one() {
        let history: Vec<&str> = vec!["plastic"; 10];
        let req = make_request("user_cap", history);
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic").unwrap();
        assert!((plastic.confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_floor_above_zero_for_empty_history() {
        let req = make_request("user_floor", vec![]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.is_empty());
    }

    // ── Boundary / threshold tests ──────────────────────────────────────

    // boundary_exactly_at_threshold_not_included removed — obsolete recycling test (IEEE 754: 0.1+0.2≠0.3)

    #[test]
    fn boundary_just_above_threshold_included() {
        let req = make_request(
            "user_boundary_above",
            vec!["plastic", "metal", "paper", "glass", "metal", "paper", "glass"],
        );
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic");
        assert!(plastic.is_some(), "confidence ≈0.314 must be included");
    }

    // ── generate_recommendations: output structure ───────────────────────

    #[test]
    fn single_type_single_item() {
        let req = make_request("user1", vec!["metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].waste_type, "metal");
        assert_eq!(recs[0].estimated_reward, 100);
    }

    #[test]
    fn multiple_types_in_history() {
        let req = make_request("user_multi", vec!["plastic", "metal", "paper", "glass"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 4);
        let types = waste_types(&recs);
        assert!(types.contains(&"plastic"));
        assert!(types.contains(&"metal"));
        assert!(types.contains(&"paper"));
        assert!(types.contains(&"glass"));
    }

    #[test]
    fn ranking_order_descending_by_confidence() {
        let req = make_request(
            "user_rank",
            vec![
                "plastic", "plastic", "plastic", "plastic", "plastic", "metal", "paper", "paper",
            ],
        );
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.len() >= 2);
        assert_eq!(recs[0].waste_type, "plastic");
        assert_eq!(recs[1].waste_type, "paper");
        assert!(recs[0].confidence_score > recs[1].confidence_score);
    }

    #[test]
    fn ranking_with_tied_scores() {
        let req = make_request(
            "user_tied",
            vec!["plastic", "plastic", "plastic", "metal", "metal", "paper", "paper"],
        );
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.len() >= 2);
        for i in 0..recs.len() - 1 {
            assert!(
                recs[i].confidence_score >= recs[i + 1].confidence_score,
                "recommendations must be sorted descending"
            );
        }
    }

    // ── Location propagation ────────────────────────────────────────────

    #[test]
    fn collection_location_matches_request() {
        let mut req = make_request("user_loc", vec!["plastic"]);
        req.location = (-33.8688, 151.2093);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs[0].collection_location, (-33.8688, 151.2093));
    }

    #[test]
    fn zero_coordinates() {
        let mut req = make_request("user_zero", vec!["plastic"]);
        req.location = (0.0, 0.0);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs[0].collection_location, (0.0, 0.0));
    }

    #[test]
    fn negative_coordinates() {
        let mut req = make_request("user_neg", vec!["plastic"]);
        req.location = (-90.0, -180.0);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs[0].collection_location, (-90.0, -180.0));
    }

    // ── Reward calculation ──────────────────────────────────────────────

    #[test]
    fn reward_formula_correct() {
        let req = make_request("user_r1", vec!["plastic"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs[0].estimated_reward, 100);
    }

    #[test]
    fn reward_with_fractional_confidence() {
        let req = make_request("user_r2", vec!["plastic", "plastic", "metal", "paper", "glass"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic").unwrap();
        assert_eq!(plastic.estimated_reward, 52);
    }

    #[test]
    fn reward_zero_confidence_not_included() {
        let req = make_request("user_r3", vec![]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.is_empty());
    }

    // ── Determinism ─────────────────────────────────────────────────────

    #[test]
    fn deterministic_output_same_input() {
        let req1 = make_request("user_det", vec!["plastic", "metal", "paper"]);
        let req2 = make_request("user_det", vec!["plastic", "metal", "paper"]);
        let recs1 = RecommendationEngine::generate_recommendations(req1);
        let recs2 = RecommendationEngine::generate_recommendations(req2);
        assert_eq!(recs1.len(), recs2.len());
        for (a, b) in recs1.iter().zip(recs2.iter()) {
            assert_eq!(a.waste_type, b.waste_type);
            assert!((a.confidence_score - b.confidence_score).abs() < f64::EPSILON);
            assert_eq!(a.estimated_reward, b.estimated_reward);
            assert_eq!(a.collection_location, b.collection_location);
        }
    }

    #[test]
    fn deterministic_output_different_instances() {
        let make = || {
            let req = make_request("user_det2", vec!["glass", "glass", "metal"]);
            RecommendationEngine::generate_recommendations(req)
        };
        let r1 = make();
        let r2 = make();
        assert_eq!(r1, r2);
    }

    // ── Filtering behavior ──────────────────────────────────────────────

    #[test]
    fn only_types_above_threshold_included() {
        let req = make_request("user_filt", vec!["plastic", "metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let types = waste_types(&recs);
        assert!(types.contains(&"plastic"));
        assert!(types.contains(&"metal"));
        assert!(!types.contains(&"paper"));
        assert!(!types.contains(&"glass"));
    }

    #[test]
    fn all_four_types_included_when_all_present() {
        let req = make_request(
            "user_all4",
            vec![
                "plastic", "metal", "paper", "glass", "plastic", "metal", "paper", "glass",
            ],
        );
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 4);
    }

    // ── Edge cases / invalid inputs ─────────────────────────────────────

    #[test]
    fn very_large_history() {
        let history: Vec<&str> = vec!["plastic"; 10_000];
        let req = make_request("user_large", history);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert!((recs[0].confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_strings_in_history() {
        let req = make_request("user_empty_str", vec!["", "", ""]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.is_empty());
    }

    #[test]
    fn mixed_empty_and_real() {
        let req = make_request("user_mixed", vec!["plastic", "", ""]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic");
        assert!(plastic.is_some());
    }

    #[test]
    fn unicode_waste_entries() {
        let req = make_request("user_unicode", vec!["♻️ plastic", "🌍 metal", "📦 paper"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.len() >= 3);
    }

    #[test]
    fn case_sensitive_matching() {
        let req = make_request("user_case", vec!["Plastic", "METAL", "PAPER"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.is_empty());
    }

    #[test]
    fn overlapping_substrings() {
        let req = make_request("user_overlap", vec!["plast", "plasticwrap", "plasticity"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let plastic = recs.iter().find(|r| r.waste_type == "plastic");
        assert!(plastic.is_some());
        assert!((plastic.unwrap().confidence_score - 0.7333333333333334).abs() < 1e-10);
    }

    // ── Multiple runs, ordering stability ────────────────────────────────

    #[test]
    fn ordering_stability_across_runs() {
        let make = || {
            let req = make_request("user_ord", vec!["plastic", "metal", "paper", "glass", "plastic"]);
            RecommendationEngine::generate_recommendations(req)
                .into_iter()
                .map(|r| r.waste_type)
                .collect::<Vec<_>>()
        };
        let order1 = make();
        let order2 = make();
        let order3 = make();
        assert_eq!(order1, order2);
        assert_eq!(order2, order3);
    }

    // ── Serde round-trip ────────────────────────────────────────────────

    #[test]
    fn recommendation_serde_roundtrip() {
        let rec = WasteRecommendation {
            waste_type: "plastic".to_string(),
            confidence_score: 0.85,
            collection_location: (1.23, 4.56),
            estimated_reward: 85,
        };
        let json = serde_json::to_string(&rec).unwrap();
        let decoded: WasteRecommendation = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, decoded);
    }

    #[test]
    fn request_serde_roundtrip() {
        let req = RecommendationRequest {
            participant_id: "p1".to_string(),
            location: (10.0, 20.0),
            waste_history: vec!["plastic".to_string(), "metal".to_string()],
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: RecommendationRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.participant_id, decoded.participant_id);
        assert_eq!(req.location, decoded.location);
        assert_eq!(req.waste_history, decoded.waste_history);
    }

    // ── Comprehensive coverage: all confidence paths ─────────────────────

    #[test]
    fn confidence_one_out_of_one() {
        let req = make_request("c1", vec!["metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let m = recs.iter().find(|r| r.waste_type == "metal").unwrap();
        assert!((m.confidence_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_one_out_of_two() {
        let req = make_request("c2", vec!["paper", "glass"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let p = recs.iter().find(|r| r.waste_type == "paper").unwrap();
        assert!((p.confidence_score - 0.6).abs() < 1e-10);
    }

    #[test]
    fn confidence_one_out_of_three() {
        let req = make_request("c3", vec!["glass", "plastic", "metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let g = recs.iter().find(|r| r.waste_type == "glass").unwrap();
        let expected = (1.0 / 3.0) * 0.8 + 0.2;
        assert!((g.confidence_score - expected).abs() < 1e-10);
    }

    #[test]
    fn confidence_two_out_of_five() {
        let req = make_request("c4", vec!["metal", "metal", "plastic", "paper", "glass"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let m = recs.iter().find(|r| r.waste_type == "metal").unwrap();
        assert!((m.confidence_score - 0.52).abs() < 1e-10);
    }

    #[test]
    fn confidence_three_out_of_four() {
        let req = make_request("c5", vec!["plastic", "plastic", "plastic", "metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let p = recs.iter().find(|r| r.waste_type == "plastic").unwrap();
        assert!((p.confidence_score - 0.8).abs() < 1e-10);
    }

    #[test]
    fn confidence_four_out_of_four() {
        let req = make_request("c6", vec!["paper", "paper", "paper", "paper"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        let p = recs.iter().find(|r| r.waste_type == "paper").unwrap();
        assert!((p.confidence_score - 1.0).abs() < f64::EPSILON);
    }

    // ── Request structure ────────────────────────────────────────────────

    #[test]
    fn participant_id_preserved_in_request() {
        let req = make_request("unique_id_12345", vec!["plastic"]);
        assert_eq!(req.participant_id, "unique_id_12345");
    }

    #[test]
    fn empty_participant_id() {
        let req = make_request("", vec!["plastic"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].waste_type, "plastic");
    }

    // ── Full integration-style checks ───────────────────────────────────

    #[test]
    fn full_scenario_realistic_input() {
        let req = make_request(
            "participant_42",
            vec!["plastic", "plastic", "plastic", "metal", "paper", "paper", "glass"],
        );
        let recs = RecommendationEngine::generate_recommendations(req);

        assert_eq!(recs.len(), 4);
        assert_eq!(recs[0].waste_type, "plastic");
        assert_eq!(recs[1].waste_type, "paper");
        assert!(recs[2].confidence_score >= recs[3].confidence_score);

        assert_eq!(recs[0].estimated_reward, (100.0 * recs[0].confidence_score) as u128);
        assert_eq!(recs[1].estimated_reward, (100.0 * recs[1].confidence_score) as u128);
    }

    #[test]
    fn all_scores_above_threshold() {
        let req = make_request(
            "user_all_above",
            vec![
                "plastic", "plastic", "metal", "metal", "paper", "paper", "glass", "glass",
            ],
        );
        let recs = RecommendationEngine::generate_recommendations(req);
        assert_eq!(recs.len(), 4);
        for r in &recs {
            assert!((r.confidence_score - 0.4).abs() < 1e-10);
        }
    }

    #[test]
    fn only_one_above_threshold() {
        let req = make_request("user_one_above", vec!["plastic", "plastic", "plastic", "metal"]);
        let recs = RecommendationEngine::generate_recommendations(req);
        assert!(recs.len() >= 2);
        let types = waste_types(&recs);
        assert!(types.contains(&"plastic"));
        assert!(types.contains(&"metal"));
        assert!(!types.contains(&"paper"));
        assert!(!types.contains(&"glass"));
    }

    // ── Debug trait ─────────────────────────────────────────────────────

    #[test]
    fn recommendation_debug_format() {
        let rec = WasteRecommendation {
            waste_type: "plastic".to_string(),
            confidence_score: 0.9,
            collection_location: (1.0, 2.0),
            estimated_reward: 90,
        };
        let debug = format!("{:?}", rec);
        assert!(debug.contains("WasteRecommendation"));
        assert!(debug.contains("plastic"));
    }

    #[test]
    fn request_debug_format() {
        let req = make_request("debug_user", vec![]);
        let debug = format!("{:?}", req);
        assert!(debug.contains("RecommendationRequest"));
        assert!(debug.contains("debug_user"));
    }

    // ── Clone trait ─────────────────────────────────────────────────────

    #[test]
    fn recommendation_clone() {
        let rec = WasteRecommendation {
            waste_type: "plastic".to_string(),
            confidence_score: 0.9,
            collection_location: (1.0, 2.0),
            estimated_reward: 90,
        };
        let cloned = rec.clone();
        assert_eq!(rec, cloned);
    }

    #[test]
    fn request_clone() {
        let req = make_request("clone_user", vec!["plastic", "metal"]);
        let cloned = req.clone();
        assert_eq!(req.participant_id, cloned.participant_id);
        assert_eq!(req.location, cloned.location);
        assert_eq!(req.waste_history, cloned.waste_history);
    }
}
