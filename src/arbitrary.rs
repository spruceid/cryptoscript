
#[cfg(test)]
pub mod test_arbitrary_defs {
  use serde::{Deserialize, Serialize};
  use serde_json::{Map, Number, Value};
  use quickcheck::{empty_shrinker, Arbitrary, Gen};

  /// Wrapped Number for Arbitrary generation
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ArbitraryNumber {
      /// Wrapped Number
      pub number: Number,
  }

  impl Arbitrary for ArbitraryNumber {
      fn arbitrary(g: &mut Gen) -> Self {
          if Arbitrary::arbitrary(g) {
              if Arbitrary::arbitrary(g) {
                  let x: u64 = Arbitrary::arbitrary(g);
                  ArbitraryNumber { number:
                      From::from(x)
                  }
              } else {
                  let x: i64 = Arbitrary::arbitrary(g);
                  ArbitraryNumber { number:
                      From::from(x)
                  }
              }
          } else {
              let x: f64 = Arbitrary::arbitrary(g);
              ArbitraryNumber { number:
                  Number::from_f64(x)
                      .unwrap_or_else(|| From::from(0u8))
              }
          }
      }

      fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
          match self.number.as_f64() {
              None => match self.number.as_u64() {
                  None => match self.number.as_i64() {
                      None => empty_shrinker(),
                      Some(self_i64) => Box::new(
                          self_i64.shrink()
                          .map(|x| ArbitraryNumber {
                              number: From::from(x),
                          })),
                  },
                  Some(self_u64) => Box::new(
                      self_u64.shrink()
                      .map(|x| ArbitraryNumber {
                          number: From::from(x),
                      })),
              },
              Some(self_f64) => Box::new(
                  self_f64.shrink()
                  .map(|x| ArbitraryNumber {
                      number: Number::from_f64(x)
                          .unwrap_or_else(|| From::from(0u8)),
                  })),
          }
      }
  }


  /// Wrapped Map, encoded as a Vec of (key, value) pairs, for Arbitrary generation
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct ArbitraryMap {
      /// Map encoded as a Vec of (key, value) pairs
      pub map: Vec<(String, Value)>,
  }

  impl From<ArbitraryMap> for Map<String, Value> {
      fn from(x: ArbitraryMap) -> Self {
          x.map.into_iter().collect()
      }
  }

  impl Arbitrary for ArbitraryMap {
      fn arbitrary(g: &mut Gen) -> Self {
          let map_vec: Vec<(String, ArbitraryValue)> = Arbitrary::arbitrary(g);
          ArbitraryMap {
              map: map_vec.into_iter().map(|x| (x.0, x.1.value)).collect(),
          }
      }

      fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
          empty_shrinker()
      }
  }


  /// Wrapped Value for Arbitrary generation
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct ArbitraryValue {
      /// Wrapped Value
      pub value: Value,
  }

  impl Arbitrary for ArbitraryValue {
      fn arbitrary(_g: &mut Gen) -> Self {
          ArbitraryValue {
              value: Value::Null,
          }
      }

      fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
          empty_shrinker()
      }
  }
}

