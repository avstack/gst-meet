// This file is copied from xmpp-parsers

// Copyright (c) 2017-2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

macro_rules! get_attr {
  ($elem:ident, $attr:tt, $type:tt) => {
    get_attr!($elem, $attr, $type, value, value.parse()?)
  };
  ($elem:ident, $attr:tt, OptionEmpty, $value:ident, $func:expr) => {
    match $elem.attr($attr) {
      Some("") => None,
      Some($value) => Some($func),
      None => None,
    }
  };
  ($elem:ident, $attr:tt, Option, $value:ident, $func:expr) => {
    match $elem.attr($attr) {
      Some($value) => Some($func),
      None => None,
    }
  };
  ($elem:ident, $attr:tt, Required, $value:ident, $func:expr) => {
    match $elem.attr($attr) {
      Some($value) => $func,
      None => {
        return Err(xmpp_parsers::Error::ParseError(concat!(
          "Required attribute '",
          $attr,
          "' missing."
        )));
      },
    }
  };
  ($elem:ident, $attr:tt, RequiredNonEmpty, $value:ident, $func:expr) => {
    match $elem.attr($attr) {
      Some("") => {
        return Err(xmpp_parsers::Error::ParseError(concat!(
          "Required attribute '",
          $attr,
          "' must not be empty."
        )));
      },
      Some($value) => $func,
      None => {
        return Err(xmpp_parsers::Error::ParseError(concat!(
          "Required attribute '",
          $attr,
          "' missing."
        )));
      },
    }
  };
  ($elem:ident, $attr:tt, Default, $value:ident, $func:expr) => {
    match $elem.attr($attr) {
      Some($value) => $func,
      None => ::std::default::Default::default(),
    }
  };
}

macro_rules! generate_attribute {
  ($(#[$meta:meta])* $elem:ident, $name:tt, {$($(#[$a_meta:meta])* $a:ident => $b:tt),+,}) => (
      generate_attribute!($(#[$meta])* $elem, $name, {$($(#[$a_meta])* $a => $b),+});
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, {$($(#[$a_meta:meta])* $a:ident => $b:tt),+,}, Default = $default:ident) => (
      generate_attribute!($(#[$meta])* $elem, $name, {$($(#[$a_meta])* $a => $b),+}, Default = $default);
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, {$($(#[$a_meta:meta])* $a:ident => $b:tt),+}) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          $(
              $(#[$a_meta])*
              $a
          ),+
      }
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<$elem, xmpp_parsers::Error> {
              Ok(match s {
                  $($b => $elem::$a),+,
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
              })
          }
      }
      impl std::fmt::Display for $elem {
          fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
              write!(fmt, "{}", match self {
                  $($elem::$a => $b),+
              })
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          fn into_attribute_value(self) -> Option<String> {
              Some(String::from(match self {
                  $($elem::$a => $b),+
              }))
          }
      }
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, {$($(#[$a_meta:meta])* $a:ident => $b:tt),+}, Default = $default:ident) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          $(
              $(#[$a_meta])*
              $a
          ),+
      }
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<$elem, xmpp_parsers::Error> {
              Ok(match s {
                  $($b => $elem::$a),+,
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
              })
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          #[allow(unreachable_patterns)]
          fn into_attribute_value(self) -> Option<String> {
              Some(String::from(match self {
                  $elem::$default => return None,
                  $($elem::$a => $b),+
              }))
          }
      }
      impl ::std::default::Default for $elem {
          fn default() -> $elem {
              $elem::$default
          }
      }
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, ($(#[$meta_symbol:meta])* $symbol:ident => $value:tt)) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          $(#[$meta_symbol])*
          $symbol,
          /// Value when absent.
          None,
      }
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<Self, xmpp_parsers::Error> {
              Ok(match s {
                  $value => $elem::$symbol,
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
              })
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          fn into_attribute_value(self) -> Option<String> {
              match self {
                  $elem::$symbol => Some(String::from($value)),
                  $elem::None => None
              }
          }
      }
      impl ::std::default::Default for $elem {
          fn default() -> $elem {
              $elem::None
          }
      }
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, bool) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          /// True value, represented by either 'true' or '1'.
          True,
          /// False value, represented by either 'false' or '0'.
          False,
      }
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<Self, xmpp_parsers::Error> {
              Ok(match s {
                  "true" | "1" => $elem::True,
                  "false" | "0" => $elem::False,
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
              })
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          fn into_attribute_value(self) -> Option<String> {
              match self {
                  $elem::True => Some(String::from("true")),
                  $elem::False => None
              }
          }
      }
      impl ::std::default::Default for $elem {
          fn default() -> $elem {
              $elem::False
          }
      }
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $type:tt, Default = $default:expr) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub struct $elem(pub $type);
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<Self, xmpp_parsers::Error> {
              Ok($elem($type::from_str(s)?))
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          fn into_attribute_value(self) -> Option<String> {
              match self {
                  $elem($default) => None,
                  $elem(value) => Some(format!("{}", value)),
              }
          }
      }
      impl ::std::default::Default for $elem {
          fn default() -> $elem {
              $elem($default)
          }
      }
  );
}

macro_rules! generate_element_enum {
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+,}) => (
      generate_element_enum!($(#[$meta])* $elem, $name, $ns, {$($(#[$enum_meta])* $enum => $enum_name),+});
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+}) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          $(
              $(#[$enum_meta])*
              $enum
          ),+
      }
      impl ::std::convert::TryFrom<xmpp_parsers::Element> for $elem {
          type Error = xmpp_parsers::Error;
          fn try_from(elem: xmpp_parsers::Element) -> Result<$elem, xmpp_parsers::Error> {
              check_ns_only!(elem, $name, $ns);
              Ok(match elem.name() {
                  $($enum_name => $elem::$enum,)+
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("This is not a ", $name, " element."))),
              })
          }
      }
      impl From<$elem> for xmpp_parsers::Element {
          fn from(elem: $elem) -> xmpp_parsers::Element {
              xmpp_parsers::Element::builder(
                  match elem {
                      $($elem::$enum => $enum_name,)+
                  },
                  $ns,
              )
                  .build()
          }
      }
  );
}

macro_rules! generate_attribute_enum {
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, $attr:tt, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+,}) => (
      generate_attribute_enum!($(#[$meta])* $elem, $name, $ns, $attr, {$($(#[$enum_meta])* $enum => $enum_name),+});
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, $attr:tt, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+}) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub enum $elem {
          $(
              $(#[$enum_meta])*
              $enum
          ),+
      }
      impl ::std::convert::TryFrom<xmpp_parsers::Element> for $elem {
          type Error = xmpp_parsers::Error;
          fn try_from(elem: xmpp_parsers::Element) -> Result<$elem, xmpp_parsers::Error> {
              check_ns_only!(elem, $name, $ns);
              Ok(match get_attr!(elem, $attr, Required) {
                  $($enum_name => $elem::$enum,)+
                  _ => return Err(xmpp_parsers::Error::ParseError(concat!("Invalid ", $name, " ", $attr, " value."))),
              })
          }
      }
      impl From<$elem> for xmpp_parsers::Element {
          fn from(elem: $elem) -> xmpp_parsers::Element {
              xmpp_parsers::Element::builder($name, $ns)
                  .attr($attr, match elem {
                       $($elem::$enum => $enum_name,)+
                   })
                   .build()
          }
      }
  );
}

macro_rules! check_self {
  ($elem:ident, $name:tt, $ns:ident) => {
    check_self!($elem, $name, $ns, $name);
  };
  ($elem:ident, $name:tt, $ns:ident, $pretty_name:tt) => {
    if !$elem.is($name, $ns) {
      return Err(xmpp_parsers::Error::ParseError(concat!(
        "This is not a ",
        $pretty_name,
        " element."
      )));
    }
  };
}

macro_rules! check_ns_only {
  ($elem:ident, $name:tt, $ns:ident) => {
    if !$elem.has_ns($ns) {
      return Err(xmpp_parsers::Error::ParseError(concat!(
        "This is not a ",
        $name,
        " element."
      )));
    }
  };
}

macro_rules! generate_empty_element {
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub struct $elem;

      impl ::std::convert::TryFrom<xmpp_parsers::Element> for $elem {
          type Error = xmpp_parsers::Error;

          fn try_from(elem: xmpp_parsers::Element) -> Result<$elem, xmpp_parsers::Error> {
              check_self!(elem, $name, $ns);
              Ok($elem)
          }
      }

      impl From<$elem> for xmpp_parsers::Element {
          fn from(_: $elem) -> xmpp_parsers::Element {
              xmpp_parsers::Element::builder($name, $ns)
                  .build()
          }
      }
  );
}

macro_rules! generate_id {
  ($(#[$meta:meta])* $elem:ident) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq, Eq, Hash)]
      pub struct $elem(pub String);
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<$elem, xmpp_parsers::Error> {
              // TODO: add a way to parse that differently when needed.
              Ok($elem(String::from(s)))
          }
      }
      impl ::minidom::IntoAttributeValue for $elem {
          fn into_attribute_value(self) -> Option<String> {
              Some(self.0)
          }
      }
  );
}

macro_rules! generate_elem_id {
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident) => (
      generate_elem_id!($(#[$meta])* $elem, $name, $ns, String);
      impl ::std::str::FromStr for $elem {
          type Err = xmpp_parsers::Error;
          fn from_str(s: &str) -> Result<$elem, xmpp_parsers::Error> {
              // TODO: add a way to parse that differently when needed.
              Ok($elem(String::from(s)))
          }
      }
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, $type:ty) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq, Eq, Hash)]
      pub struct $elem(pub $type);
      impl ::std::convert::TryFrom<xmpp_parsers::Element> for $elem {
          type Error = xmpp_parsers::Error;
          fn try_from(elem: xmpp_parsers::Element) -> Result<$elem, xmpp_parsers::Error> {
              check_self!(elem, $name, $ns);
              // TODO: add a way to parse that differently when needed.
              Ok($elem(elem.text().parse()?))
          }
      }
      impl From<$elem> for xmpp_parsers::Element {
          fn from(elem: $elem) -> xmpp_parsers::Element {
              xmpp_parsers::Element::builder($name, $ns)
                  .append(elem.0.to_string())
                  .build()
          }
      }
  );
}

macro_rules! decl_attr {
  (OptionEmpty, $type:ty) => (
      Option<$type>
  );
  (Option, $type:ty) => (
      Option<$type>
  );
  (Required, $type:ty) => (
      $type
  );
  (RequiredNonEmpty, $type:ty) => (
      $type
  );
  (Default, $type:ty) => (
      $type
  );
}

macro_rules! start_decl {
  (Vec, $type:ty) => (
      Vec<$type>
  );
  (Option, $type:ty) => (
      Option<$type>
  );
  (Required, $type:ty) => (
      $type
  );
  (Present, $type:ty) => (
      bool
  );
}

macro_rules! start_parse_elem {
  ($temp:ident: Vec) => {
    let mut $temp = Vec::new();
  };
  ($temp:ident: Option) => {
    let mut $temp = None;
  };
  ($temp:ident: Required) => {
    let mut $temp = None;
  };
  ($temp:ident: Present) => {
    let mut $temp = false;
  };
}

macro_rules! do_parse {
  ($elem:ident, Element) => {
    $elem.clone()
  };
  ($elem:ident, String) => {
    $elem.text()
  };
  ($elem:ident, $constructor:ident) => {
    $constructor::try_from($elem.clone())?
  };
}

macro_rules! do_parse_elem {
  ($temp:ident: Vec = $constructor:ident => $elem:ident, $name:tt, $parent_name:tt) => {
    $temp.push(do_parse!($elem, $constructor));
  };
  ($temp:ident: Option = $constructor:ident => $elem:ident, $name:tt, $parent_name:tt) => {
    if $temp.is_some() {
      return Err(xmpp_parsers::Error::ParseError(concat!(
        "Element ",
        $parent_name,
        " must not have more than one ",
        $name,
        " child."
      )));
    }
    $temp = Some(do_parse!($elem, $constructor));
  };
  ($temp:ident: Required = $constructor:ident => $elem:ident, $name:tt, $parent_name:tt) => {
    if $temp.is_some() {
      return Err(xmpp_parsers::Error::ParseError(concat!(
        "Element ",
        $parent_name,
        " must not have more than one ",
        $name,
        " child."
      )));
    }
    $temp = Some(do_parse!($elem, $constructor));
  };
  ($temp:ident: Present = $constructor:ident => $elem:ident, $name:tt, $parent_name:tt) => {
    if $temp {
      return Err(xmpp_parsers::Error::ParseError(concat!(
        "Element ",
        $parent_name,
        " must not have more than one ",
        $name,
        " child."
      )));
    }
    $temp = true;
  };
}

macro_rules! finish_parse_elem {
  ($temp:ident: Vec = $name:tt, $parent_name:tt) => {
    $temp
  };
  ($temp:ident: Option = $name:tt, $parent_name:tt) => {
    $temp
  };
  ($temp:ident: Required = $name:tt, $parent_name:tt) => {
    $temp.ok_or(xmpp_parsers::Error::ParseError(concat!(
      "Missing child ",
      $name,
      " in ",
      $parent_name,
      " element."
    )))?
  };
  ($temp:ident: Present = $name:tt, $parent_name:tt) => {
    $temp
  };
}

macro_rules! generate_serialiser {
  ($builder:ident, $parent:ident, $elem:ident, Required, String, ($name:tt, $ns:ident)) => {
    $builder.append(
      xmpp_parsers::Element::builder($name, $ns).append(::minidom::Node::Text($parent.$elem)),
    )
  };
  ($builder:ident, $parent:ident, $elem:ident, Option, String, ($name:tt, $ns:ident)) => {
    $builder.append_all(
      $parent
        .$elem
        .map(|elem| xmpp_parsers::Element::builder($name, $ns).append(::minidom::Node::Text(elem))),
    )
  };
  ($builder:ident, $parent:ident, $elem:ident, Option, $constructor:ident, ($name:tt, *)) => {
    $builder.append_all(
      $parent
        .$elem
        .map(|elem| ::minidom::Node::Element(xmpp_parsers::Element::from(elem))),
    )
  };
  ($builder:ident, $parent:ident, $elem:ident, Option, $constructor:ident, ($name:tt, $ns:ident)) => {
    $builder.append_all(
      $parent
        .$elem
        .map(|elem| ::minidom::Node::Element(xmpp_parsers::Element::from(elem))),
    )
  };
  ($builder:ident, $parent:ident, $elem:ident, Vec, $constructor:ident, ($name:tt, $ns:ident)) => {
    $builder.append_all($parent.$elem.into_iter())
  };
  ($builder:ident, $parent:ident, $elem:ident, Present, $constructor:ident, ($name:tt, $ns:ident)) => {
    $builder.append(::minidom::Node::Element(
      xmpp_parsers::Element::builder($name, $ns).build(),
    ))
  };
  ($builder:ident, $parent:ident, $elem:ident, $_:ident, $constructor:ident, ($name:tt, $ns:ident)) => {
    $builder.append(::minidom::Node::Element(xmpp_parsers::Element::from(
      $parent.$elem,
    )))
  };
}

macro_rules! generate_child_test {
  ($child:ident, $name:tt, *) => {
    $child.is($name, ::minidom::NSChoice::Any)
  };
  ($child:ident, $name:tt, $ns:tt) => {
    $child.is($name, $ns)
  };
}

macro_rules! generate_element {
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_action:tt<$attr_type:ty> = $attr_name:tt),+,]) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [$($(#[$attr_meta])* $attr: $attr_action<$attr_type> = $attr_name),*], children: []);
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_action:tt<$attr_type:ty> = $attr_name:tt),+]) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [$($(#[$attr_meta])* $attr: $attr_action<$attr_type> = $attr_name),*], children: []);
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, children: [$($(#[$child_meta:meta])* $child_ident:ident: $coucou:tt<$child_type:ty> = ($child_name:tt, $child_ns:tt) => $child_constructor:ident),*]) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [], children: [$($(#[$child_meta])* $child_ident: $coucou<$child_type> = ($child_name, $child_ns) => $child_constructor),*]);
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_action:tt<$attr_type:ty> = $attr_name:tt),*,], children: [$($(#[$child_meta:meta])* $child_ident:ident: $coucou:tt<$child_type:ty> = ($child_name:tt, $child_ns:tt) => $child_constructor:ident),*]) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [$($(#[$attr_meta])* $attr: $attr_action<$attr_type> = $attr_name),*], children: [$($(#[$child_meta])* $child_ident: $coucou<$child_type> = ($child_name, $child_ns) => $child_constructor),*]);
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, text: ($(#[$text_meta:meta])* $text_ident:ident: $codec:ident < $text_type:ty >)) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [], children: [], text: ($(#[$text_meta])* $text_ident: $codec<$text_type>));
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_action:tt<$attr_type:ty> = $attr_name:tt),+], text: ($(#[$text_meta:meta])* $text_ident:ident: $codec:ident < $text_type:ty >)) => (
      generate_element!($(#[$meta])* $elem, $name, $ns, attributes: [$($(#[$attr_meta])* $attr: $attr_action<$attr_type> = $attr_name),*], children: [], text: ($(#[$text_meta])* $text_ident: $codec<$text_type>));
  );
  ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:ident, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_action:tt<$attr_type:ty> = $attr_name:tt),*], children: [$($(#[$child_meta:meta])* $child_ident:ident: $coucou:tt<$child_type:ty> = ($child_name:tt, $child_ns:tt) => $child_constructor:ident),*] $(, text: ($(#[$text_meta:meta])* $text_ident:ident: $codec:ident < $text_type:ty >))*) => (
      $(#[$meta])*
      #[derive(Debug, Clone, PartialEq)]
      pub struct $elem {
          $(
              $(#[$attr_meta])*
              pub $attr: decl_attr!($attr_action, $attr_type),
          )*
          $(
              $(#[$child_meta])*
              pub $child_ident: start_decl!($coucou, $child_type),
          )*
          $(
              $(#[$text_meta])*
              pub $text_ident: $text_type,
          )*
      }

      impl ::std::convert::TryFrom<xmpp_parsers::Element> for $elem {
          type Error = xmpp_parsers::Error;

          fn try_from(elem: xmpp_parsers::Element) -> Result<$elem, xmpp_parsers::Error> {
              check_self!(elem, $name, $ns);
              $(
                  start_parse_elem!($child_ident: $coucou);
              )*
              for _child in elem.children() {
                  $(
                  if generate_child_test!(_child, $child_name, $child_ns) {
                      do_parse_elem!($child_ident: $coucou = $child_constructor => _child, $child_name, $name);
                      continue;
                  }
                  )*
              }
              Ok($elem {
                  $(
                      $attr: get_attr!(elem, $attr_name, $attr_action),
                  )*
                  $(
                      $child_ident: finish_parse_elem!($child_ident: $coucou = $child_name, $name),
                  )*
                  $(
                      $text_ident: $codec::decode(&elem.text())?,
                  )*
              })
          }
      }

      impl From<$elem> for xmpp_parsers::Element {
          fn from(elem: $elem) -> xmpp_parsers::Element {
              let mut builder = xmpp_parsers::Element::builder($name, $ns);
              $(
                  builder = builder.attr($attr_name, elem.$attr);
              )*
              $(
                  builder = generate_serialiser!(builder, elem, $child_ident, $coucou, $child_constructor, ($child_name, $child_ns));
              )*
              $(
                  builder = builder.append_all($codec::encode(&elem.$text_ident).map(::minidom::Node::Text).into_iter());
              )*

              builder.build()
          }
      }
  );
}

#[cfg(test)]
macro_rules! assert_size (
  ($t:ty, $sz:expr) => (
      assert_eq!(::std::mem::size_of::<$t>(), $sz);
  );
);

// TODO: move that to src/pubsub/mod.rs, once we figure out how to use macros from there.
macro_rules! impl_pubsub_item {
  ($item:ident, $ns:ident) => {
    impl ::std::convert::TryFrom<xmpp_parsers::Element> for $item {
      type Error = Error;

      fn try_from(elem: xmpp_parsers::Element) -> Result<$item, Error> {
        check_self!(elem, "item", $ns);
        let mut payloads = elem.children().cloned().collect::<Vec<_>>();
        let payload = payloads.pop();
        if !payloads.is_empty() {
          return Err(Error::ParseError(
            "More than a single payload in item element.",
          ));
        }
        Ok($item(xmpp_parsers::pubsub::Item {
          id: get_attr!(elem, "id", Option),
          publisher: get_attr!(elem, "publisher", Option),
          payload,
        }))
      }
    }

    impl From<$item> for xmpp_parsers::Element {
      fn from(item: $item) -> xmpp_parsers::Element {
        xmpp_parsers::Element::builder("item", $ns)
          .attr("id", item.0.id)
          .attr("publisher", item.0.publisher)
          .append_all(item.0.payload)
          .build()
      }
    }

    impl ::std::ops::Deref for $item {
      type Target = xmpp_parsers::pubsub::Item;

      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl ::std::ops::DerefMut for $item {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }
  };
}
