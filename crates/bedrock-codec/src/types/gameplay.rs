use crate::types::enums::game_rule_type as grt;
use crate::types::primitives::{ArrayI32, ArrayU32};
use crate::{Codec, Error, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq)]
pub enum GameRuleValue {
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
}

impl GameRuleValue {
    pub fn type_id(&self) -> u32 {
        match self {
            GameRuleValue::Null => grt::NULL,
            GameRuleValue::Bool(_) => grt::BOOL,
            GameRuleValue::Int(_) => grt::INT,
            GameRuleValue::Float(_) => grt::FLOAT,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameRule {
    pub name: String,
    pub editable: bool,
    pub value: GameRuleValue,
}

pub struct GameRules;

impl Codec for GameRules {
    type Value = Vec<GameRule>;

    fn read(r: &mut Reader<'_>) -> Result<Vec<GameRule>> {
        let count = r.read_count()?;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let name = r.read_string()?;
            let editable = r.read_bool()?;
            let type_id = r.read_uvarint()?;
            let value = match type_id {
                grt::BOOL => GameRuleValue::Bool(r.read_bool()?),
                grt::INT => GameRuleValue::Int(r.read_uvarint()? as i32),
                grt::FLOAT => GameRuleValue::Float(r.read_f32_le()?),
                other => {
                    return Err(Error::BadDiscriminant {
                        what: "game rule type",
                        value: other as i64,
                    })
                }
            };
            out.push(GameRule {
                name,
                editable,
                value,
            });
        }
        Ok(out)
    }

    fn write(w: &mut Writer, v: &Vec<GameRule>) {
        let kept = v.iter().filter(|rule| rule.value != GameRuleValue::Null);
        w.write_count(kept.clone().count());
        for rule in kept {
            w.write_string(&rule.name);
            w.write_bool(rule.editable);
            w.write_uvarint(rule.value.type_id());
            match &rule.value {
                GameRuleValue::Null => unreachable!("filtered above"),
                GameRuleValue::Bool(x) => w.write_bool(*x),
                GameRuleValue::Int(x) => w.write_uvarint(*x as u32),
                GameRuleValue::Float(x) => w.write_f32_le(*x),
            }
        }
    }
}

pub struct GameRulesV2168;

impl Codec for GameRulesV2168 {
    type Value = Vec<GameRule>;

    fn read(r: &mut Reader<'_>) -> Result<Vec<GameRule>> {
        let count = r.read_count()?;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let name = r.read_string()?;
            let editable = r.read_bool()?;
            let type_id = r.read_uvarint()?;
            let value = match type_id {
                grt::NULL => GameRuleValue::Null,
                grt::BOOL => GameRuleValue::Bool(r.read_bool()?),
                grt::INT => GameRuleValue::Int(r.read_u32_le()? as i32),
                grt::FLOAT => GameRuleValue::Float(r.read_f32_le()?),
                other => {
                    return Err(Error::BadDiscriminant {
                        what: "game rule type",
                        value: other as i64,
                    })
                }
            };
            out.push(GameRule {
                name,
                editable,
                value,
            });
        }
        Ok(out)
    }

    fn write(w: &mut Writer, v: &Vec<GameRule>) {
        w.write_count(v.len());
        for rule in v {
            w.write_string(&rule.name);
            w.write_bool(rule.editable);
            w.write_uvarint(rule.value.type_id());
            match &rule.value {
                GameRuleValue::Null => {}
                GameRuleValue::Bool(x) => w.write_bool(*x),
                GameRuleValue::Int(x) => w.write_u32_le(*x as u32),
                GameRuleValue::Float(x) => w.write_f32_le(*x),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    pub name: String,
    pub enabled: bool,
}

pub struct ExperimentEntry;

impl Codec for ExperimentEntry {
    type Value = Experiment;

    fn read(r: &mut Reader<'_>) -> Result<Experiment> {
        Ok(Experiment {
            name: r.read_string()?,
            enabled: r.read_bool()?,
        })
    }

    fn write(w: &mut Writer, v: &Experiment) {
        w.write_string(&v.name);
        w.write_bool(v.enabled);
    }
}

pub type Experiments = ArrayU32<ExperimentEntry>;
pub type ExperimentsV860 = ArrayI32<ExperimentEntry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_rule_int_is_an_unsigned_varint() {
        let rules = vec![GameRule {
            name: "randomTickSpeed".into(),
            editable: true,
            value: GameRuleValue::Int(3),
        }];
        let mut w = Writer::new();
        GameRules::write(&mut w, &rules);
        let bytes = w.into_vec();
        assert_eq!(*bytes.last().unwrap(), 0x03);
        let mut r = Reader::new(&bytes);
        assert_eq!(GameRules::read(&mut r).unwrap(), rules);
    }

    #[test]
    fn an_int_rule_keeps_its_value_across_the_v2168_boundary() {
        let rules = vec![GameRule {
            name: "randomTickSpeed".into(),
            editable: true,
            value: GameRuleValue::Int(3),
        }];

        let mut w = Writer::new();
        GameRules::write(&mut w, &rules);
        let v1001 = w.into_vec();

        let mut r = Reader::new(&v1001);
        let decoded = GameRules::read(&mut r).unwrap();
        let mut w = Writer::new();
        GameRulesV2168::write(&mut w, &decoded);
        let v2168 = w.into_vec();

        let mut r = Reader::new(&v2168);
        assert_eq!(GameRulesV2168::read(&mut r).unwrap(), rules);
        assert_eq!(&v2168[v2168.len() - 4..], &[3, 0, 0, 0]);
    }

    #[test]
    fn v2168_ints_are_fixed_width_not_zigzag() {
        let rules = vec![GameRule {
            name: "randomTickSpeed".into(),
            editable: true,
            value: GameRuleValue::Int(-1),
        }];
        let mut w = Writer::new();
        GameRulesV2168::write(&mut w, &rules);
        let bytes = w.into_vec();
        assert_eq!(&bytes[bytes.len() - 4..], &[0xFF, 0xFF, 0xFF, 0xFF]);
        let mut r = Reader::new(&bytes);
        assert_eq!(GameRulesV2168::read(&mut r).unwrap(), rules);
    }

    #[test]
    fn null_rules_survive_v2168_but_are_dropped_going_down() {
        let rules = vec![
            GameRule {
                name: "a".into(),
                editable: true,
                value: GameRuleValue::Null,
            },
            GameRule {
                name: "b".into(),
                editable: false,
                value: GameRuleValue::Bool(true),
            },
        ];
        let mut new = Writer::new();
        GameRulesV2168::write(&mut new, &rules);
        let bytes = new.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(GameRulesV2168::read(&mut r).unwrap(), rules);

        let mut old = Writer::new();
        GameRules::write(&mut old, &rules);
        let bytes = old.into_vec();
        assert_eq!(bytes[0], 1);
        let mut r = Reader::new(&bytes);
        assert_eq!(GameRules::read(&mut r).unwrap(), rules[1..].to_vec());
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn experiment_count_widths_differ() {
        let exps = vec![Experiment {
            name: "x".into(),
            enabled: true,
        }];
        let mut a = Writer::new();
        Experiments::write(&mut a, &exps);
        let mut b = Writer::new();
        ExperimentsV860::write(&mut b, &exps);
        assert_eq!(a.len(), b.len());
        assert_eq!(a.as_slice(), b.as_slice());
    }
}
