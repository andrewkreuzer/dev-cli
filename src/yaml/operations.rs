use log::info;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    op: String,
    path: String,
    from: Option<String>,
    value: Option<String>,
}

impl Operation {
    pub fn new(op: String, path: String, from: Option<String>, value: Option<String>) -> Self {
        Operation { op, path, from, value }
    }

    pub fn run(&self, value: &mut serde_yaml::Value) {
        match self.op.as_str() {
            "add" => self.add(value, self.path.as_str(), self.value.as_ref().unwrap().as_str()),
            "remove" => self.remove(value),
            "replace" => self.replace(value),
            // "move" => self.move_(value),
            // "copy" => self.copy(value),
            // "test" => self.test(value),
            _ => panic!("invalid operation"),
        }
    }

    fn add(&self, value: &mut serde_yaml::Value, path: &str, value_str: &str) {
        let mut path_iter = path.split('/');
        path_iter.next();
        let mut current = value;
        let last = path_iter.next_back().unwrap();
        for p in path_iter {
            current = current
                .as_mapping_mut()
                .unwrap()
                .get_mut(&serde_yaml::Value::String(p.to_string()))
                .unwrap();
        }
        let value = serde_yaml::from_str(value_str).unwrap();
        current
            .as_mapping_mut()
            .unwrap()
            .insert(serde_yaml::Value::String(last.to_string()), value);
    }

    fn remove(&self, value: &mut serde_yaml::Value) {
        let mut path_iter = self.path.split('/');
        path_iter.next();
        let mut current = value;
        let last = path_iter.next_back().unwrap();
        for p in path_iter {
            current = current
                .as_mapping_mut()
                .unwrap()
                .get_mut(serde_yaml::Value::String(p.to_string()))
                .unwrap();
        }
        current.as_mapping_mut().unwrap().remove(serde_yaml::Value::String(last.to_string()));
    }

    fn replace(&self, value: &mut serde_yaml::Value) {
        let mut path_iter = self.path.split('/');
        path_iter.next();
        let mut current = value;
        let last = path_iter.next_back().unwrap();
        for p in path_iter {
            current = current
                .as_mapping_mut()
                .unwrap()
                .get_mut(serde_yaml::Value::String(p.to_string()))
                .unwrap();
        }
        let value = serde_yaml::from_str(self.value.as_ref().unwrap()).unwrap();
        current
            .as_mapping_mut()
            .unwrap()
            .insert(serde_yaml::Value::String(last.to_string()), value);
    }
}

pub fn walk(value: &mut Value, target: &str, _path: &str) {
    match value {
        Value::Null => (),
        Value::Bool(_bool) => (),
        Value::Number(_num) => (),
        Value::String(string) => {
            if string == target {
                *string = "something else".into();
            }
        }
        Value::Sequence(sequence) => {
            for v in sequence.iter_mut() {
                walk(v, target, _path);
            }
        }
        Value::Mapping(mapping) => {
            for map in mapping.iter_mut() {
                if map.0 == target {
                    info!("found key which matches target")
                }

                walk(map.1, target, _path);
            }
        }
        Value::Tagged(_tagged_value) => (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut value = serde_yaml::from_str(
r#"
a:
  b:
    c: 1
"#,
        ).unwrap();
        let op = Operation::new("add".into(), "/a/b/d".into(), None, Some("2".into()));
        op.run(&mut value);
        assert_eq!(
            serde_yaml::to_string(&value).unwrap(),
r#"
a:
  b:
    c: 1
    d: 2
"#.trim_start(),
        );
    }

    #[test]
    fn remove() {
        let mut value = serde_yaml::from_str(
r#"
a:
  b:
    c: 1
    d: 2
"#,
        ).unwrap();
        let op = Operation::new("remove".into(), "/a/b/d".into(), None, None);
        op.run(&mut value);
        assert_eq!(
            serde_yaml::to_string(&value).unwrap(),
r#"
a:
  b:
    c: 1
"#.trim_start(),
        );
    }

    #[test]
    fn replace() {
        let mut value = serde_yaml::from_str(
r#"
a:
  b:
    c: 1
    d: 2
"#,
        ).unwrap();
        let op = Operation::new("replace".into(), "/a/b/d".into(), None, Some("3".into()));
        op.run(&mut value);
        assert_eq!(
            serde_yaml::to_string(&value).unwrap(),
r#"
a:
  b:
    c: 1
    d: 3
"#.trim_start(),
        );
    }
}
