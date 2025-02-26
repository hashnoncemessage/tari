//  Copyright 2021, The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::{
    convert::TryFrom,
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};

use crate::ConfigurationError;

static CURRENT_NETWORK: OnceLock<Network> = OnceLock::new();

/// Represents the available Tari p2p networks. Only nodes with matching byte values will be able to connect, so these
/// should never be changed once released.
#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Network {
    MainNet = 0xaa,
    TestNet = 0xbb,
    LocalNet = 0xcc,
}

impl Network {
    pub fn get_current_or_user_setting_or_default() -> Self {
        match CURRENT_NETWORK.get() {
            Some(&network) => network,
            None => {
                // Check to see if the network has been set by the environment, otherwise use the default
                match std::env::var("TARI_NETWORK") {
                    Ok(network) => Network::from_str(network.as_str()).unwrap_or(Network::default()),
                    Err(_) => Network::default(),
                }
            },
        }
    }

    pub fn set_current(network: Network) -> Result<(), Network> {
        CURRENT_NETWORK.set(network)
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    pub const fn as_key_str(self) -> &'static str {
        #[allow(clippy::enum_glob_use)]
        use Network::*;
        match self {
            MainNet => "mainnet",
            TestNet => "testnet",
            LocalNet => "localnet",
        }
    }
}

/// The default network for all applications
impl Default for Network {
    #[cfg(tari_target_network_mainnet)]
    fn default() -> Self {
        match std::env::var("TARI_NETWORK") {
            Ok(network) => Network::from_str(network.as_str()).unwrap_or(Network::MainNet),
            Err(_) => panic!("invalid network"),
        }
    }

    #[cfg(not(any(tari_target_network_mainnet, tari_target_network_nextnet)))]
    fn default() -> Self {
        Network::TestNet
    }
}

impl FromStr for Network {
    type Err = ConfigurationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        #[allow(clippy::enum_glob_use)]
        use Network::*;
        match value.to_lowercase().as_str() {
            "mainnet" => Ok(MainNet),
            "testnet" => Ok(TestNet),
            "localnet" => Ok(LocalNet),
            invalid => Err(ConfigurationError::new(
                "network",
                Some(value.to_string()),
                format!("Invalid network option: {}", invalid),
            )),
        }
    }
}
impl TryFrom<String> for Network {
    type Error = ConfigurationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(value.as_str())
    }
}

impl From<Network> for String {
    fn from(n: Network) -> Self {
        n.to_string()
    }
}

impl TryFrom<u8> for Network {
    type Error = ConfigurationError;

    fn try_from(v: u8) -> Result<Self, ConfigurationError> {
        match v {
            x if x == Network::MainNet as u8 => Ok(Network::MainNet),
            x if x == Network::TestNet as u8 => Ok(Network::TestNet),
            x if x == Network::LocalNet as u8 => Ok(Network::LocalNet),
            _ => Err(ConfigurationError::new(
                "network",
                Some(v.to_string()),
                format!("Invalid network option: {}", v),
            )),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.as_key_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn network_bytes() {
        // get networks
        let mainnet = Network::MainNet;
        let testnet = Network::TestNet;
        let localnet = Network::LocalNet;

        // test .as_byte()
        assert_eq!(mainnet.as_byte(), 0xaa_u8);
        assert_eq!(testnet.as_byte(), 0xbb_u8);
        assert_eq!(localnet.as_byte(), 0xcc_u8);

        // test .as_key_str()
        assert_eq!(mainnet.as_key_str(), "mainnet");
        assert_eq!(testnet.as_key_str(), "testnet");
        assert_eq!(localnet.as_key_str(), "localnet");
    }

    #[test]
    fn network_default() {
        let network = Network::default();
        #[cfg(tari_target_network_mainnet)]
        assert!(matches!(network, Network::MainNet));
        #[cfg(not(any(tari_target_network_mainnet)))]
        assert_eq!(network, Network::TestNet);
    }

    #[test]
    fn network_from_str() {
        // test .from_str()
        assert_eq!(Network::from_str("mainnet").unwrap(), Network::MainNet);
        assert_eq!(Network::from_str("testnet").unwrap(), Network::TestNet);
        assert_eq!(Network::from_str("localnet").unwrap(), Network::LocalNet);
        // catch error case
        let err_network = Network::from_str("invalid network");
        assert!(err_network.is_err());
    }

    #[test]
    fn network_from_byte() {
        assert_eq!(Network::try_from(0xaa).unwrap(), Network::MainNet);
        assert_eq!(Network::try_from(0xbb).unwrap(), Network::TestNet);
        assert_eq!(Network::try_from(0xcc).unwrap(), Network::LocalNet);
    }
}
