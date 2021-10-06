// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_empty_element!(
    /// Stream:feature sent by the server to advertise it supports CSI.
    Feature,
    "csi",
    CSI
);

generate_empty_element!(
    /// Client indicates it is inactive.
    Inactive,
    "inactive",
    CSI
);

generate_empty_element!(
    /// Client indicates it is active again.
    Active,
    "active",
    CSI
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn test_size() {
        assert_size!(Feature, 0);
        assert_size!(Inactive, 0);
        assert_size!(Active, 0);
    }

    #[test]
    fn parsing() {
        let elem: Element = "<csi xmlns='urn:xmpp:csi:0'/>".parse().unwrap();
        Feature::try_from(elem).unwrap();

        let elem: Element = "<inactive xmlns='urn:xmpp:csi:0'/>".parse().unwrap();
        Inactive::try_from(elem).unwrap();

        let elem: Element = "<active xmlns='urn:xmpp:csi:0'/>".parse().unwrap();
        Active::try_from(elem).unwrap();
    }

    #[test]
    fn serialising() {
        let elem: Element = Feature.into();
        assert!(elem.is("csi", ns::CSI));

        let elem: Element = Inactive.into();
        assert!(elem.is("inactive", ns::CSI));

        let elem: Element = Active.into();
        assert!(elem.is("active", ns::CSI));
    }
}
