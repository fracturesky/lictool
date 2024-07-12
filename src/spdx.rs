use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use serde::Deserialize;
extern crate reqwest;
use std::fmt::Display;

use color_print::{cformat, cstr};

use crate::{
    consts::{EMAIL, OWNER, REPO, YEAR},
    util::{cache::http_cache_dir, errors::LictoolResult},
};

const SPDX_BASE_URL: &str = "https://spdx.org";

// const SPDX_LICENSES_URL: Url = "https://spdx.org/licenses/licenses.json";

/// A struct representing a collection of software licenses.
///
/// This struct holds a vector of `License` objects, each containing
/// details about individual software licenses.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Licenses {
    /// A vector of `License` structs representing the licenses.
    #[serde(rename = "licenses")]
    pub body: Vec<License>,
}

impl Licenses {
    pub async fn new() -> LictoolResult<Self> {
        fetch_licenses(SPDX_BASE_URL).await
    }

    /// Filters the licenses based on specified criteria.
    ///
    /// This function returns a vector of references to `License`
    /// objects that match the given criteria for deprecation
    /// status, support status, OSI approval, and FSF Libre approval.
    ///
    /// # Arguments
    ///
    /// * `deprecated` - A boolean indicating whether to include
    ///   deprecated licenses.
    /// * `supported` - A boolean indicating whether to include
    ///   supported licenses.
    /// * `osi_approved` - A boolean indicating whether to include OSI
    ///   approved licenses.
    /// * `fsf_libre` - A boolean indicating whether to include FSF
    ///   Libre approved licenses.
    ///
    /// # Returns
    ///
    /// A vector of references to `License` structs that meet the
    /// specified criteria.
    ///
    /// # Example
    ///
    /// ```
    /// let licenses = Licenses { /* initialize fields */ };
    /// let filtered = licenses.filter_by(true, true, false, false);
    /// println!("{:?}", filtered);
    /// ```
    pub fn filter_by(
        &self,
        deprecated: bool,
        supported: bool,
        osi_approved: bool,
        fsf_libre: bool,
    ) -> Vec<&License> {
        self.body
            .iter()
            .filter(|license| {
                let mut result = true;
                if deprecated {
                    result = result && license.is_deprecated_license_id == deprecated;
                }
                if supported {
                    result = result && license.is_deprecated_license_id != supported;
                }
                if osi_approved {
                    result = result && license.is_osi_approved == osi_approved;
                }
                if fsf_libre {
                    result = result && license.is_fsf_libre == Some(fsf_libre);
                }
                result
            })
            .collect()
    }
}

/// Asynchronously fetches licenses from a given base URL.
///
/// # Parameters
/// - `base_url`: The base URL as a string-like type.
///
/// # Returns
/// - `LictoolResult<Licenses>`: The result containing the licenses or
///   an error.
async fn fetch_licenses<S: Into<String>>(base_url: S) -> LictoolResult<Licenses> {
    let client = ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: http_cache_dir(),
            },
            options: HttpCacheOptions::default(),
        }))
        .build();
    let res = client
        .get(format!("{}{}", base_url.into(), "/licenses/licenses.json"))
        .send()
        .await?
        .json::<Licenses>()
        .await?;
    Ok(res)
}

/// Displays the IDs of licenses.
///
/// This function sorts the given slice of licenses by their
/// deprecation status and then prints the ID of each license.
///
/// # Arguments
///
/// * `licenses` - A mutable slice of references to `License` objects.
///
/// # Returns
///
/// A `LictoolResult` indicating the success or failure of the
/// operation.
///
/// # Example
///
/// ```
/// let mut licenses = vec![&license1, &license2];
/// display_license_ids(&mut licenses)?;
/// ```
pub(crate) fn display_license_ids(licenses: &mut [&License]) -> LictoolResult<()> {
    licenses.sort_by_key(|license| license.is_deprecated_license_id);
    licenses
        .iter()
        .for_each(|license| println!("{}", license.color_id()));
    Ok(())
}

/// A struct representing the details of a software license.
///
/// This struct is used to hold various information about a license,
/// including its text, ID, name, and other related metadata.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseDetails {
    /// A boolean indicating if the license ID is deprecated.
    pub is_deprecated_license_id: bool,
    /// A string containing the full text of the license.
    pub license_text: String,
    /// The name of the license.
    pub name: String,
    /// Optional comments about the license.
    pub license_comments: Option<String>,
    /// The unique identifier of the license.
    pub license_id: String,
    /// A list of URLs for additional information about the license.
    pub see_also: Vec<String>,
    /// A boolean indicating if the license is approved by the OSI.
    pub is_osi_approved: bool,
    /// An optional boolean indicating if the license is approved by
    /// the FSF.
    pub is_fsf_libre: Option<bool>,
    /// An optional string indicating the version in which the license
    /// was deprecated.
    pub deprecated_version: Option<String>,
}

impl Display for LicenseDetails {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let mut output = String::new();
        let term_width = termion::terminal_size().unwrap().0 as usize;
        let padding = (term_width - self.name.len()) / 2;
        write!(f, "{:width$}", "", width = padding)?;
        writeln!(f, "{}", cformat!("«<s>{}</>»", self.name))?;
        output.push_str(&cformat!(
            "<s>Reference:</> <u>https://spdx.org/licenses/{}.html</>\n",
            self.license_id
        ));
        output.push_str(&cformat!("<s>License ID:</> {}\n", self.license_id));
        if let Some(ref comments) = self.license_comments {
            output.push_str(&cformat!("<s>License Comments:</> {}\n", comments));
        }
        output.push_str(&cformat!("<s>See Also:</>\n"));
        for link in &self.see_also {
            output.push_str(&cformat!("  - <u>{}</>\n", link));
        }
        output.push_str(&cformat!(
            "<s>Is Supported License ID:</> {}\n",
            (!self.is_deprecated_license_id).as_checkbox()
        ));
        output.push_str(&cformat!(
            "<s>Is OSI Approved:</> {}",
            self.is_osi_approved.as_checkbox()
        ));
        if let Some(is_fsf_libre) = self.is_fsf_libre {
            output.push_str(&cformat!(
                "\n<s>Is FSF Free/Libre:</> {}",
                is_fsf_libre.as_checkbox()
            ));
        }
        if let Some(ref deprecated_version) = self.deprecated_version {
            output.push_str(&cformat!(
                "\n<s>Deprecated Version:</> {}",
                deprecated_version
            ));
        }
        write!(f, "{}", output)
    }
}

impl LicenseDetails {
    /// Checks if the license text contains any year-related keywords.
    pub fn has_year(&self) -> bool {
        YEAR.iter().any(|&word| self.license_text.contains(word))
    }

    /// Checks if the license text contains any owner-related
    /// keywords.
    pub fn has_owner(&self) -> bool {
        OWNER.iter().any(|&word| self.license_text.contains(word))
    }

    /// Checks if the license text contains any repository-related
    /// keywords.
    pub fn has_repo(&self) -> bool {
        REPO.iter().any(|&word| self.license_text.contains(word))
    }

    /// Checks if the license text contains any email-related
    /// keywords.
    pub fn has_email(&self) -> bool {
        EMAIL.iter().any(|&word| self.license_text.contains(word))
    }
}

/// An interface for representing a boolean value as a checkbox.
///
/// This trait provides a method to convert a boolean value into
/// a string representation of a checkbox, typically for display
/// purposes.
///
/// # Example
///
/// ```
/// let checked = true;
/// println!("{}", checked.as_checkbox()); // Output: <green, bold>󰄲</>
///
/// let unchecked = false;
/// println!("{}", unchecked.as_checkbox()); // Output: <red, bold>󰅗</>
/// ```
trait Checkbox {
    fn as_checkbox(&self) -> &str;
}

impl Checkbox for bool {
    fn as_checkbox(&self) -> &str {
        match self {
            true => cstr!("<green, bold>󰄲</>"),
            false => cstr!("<red, bold>󰅗</>"),
        }
    }
}

/// Fetches license details from a given URL.
///
/// This asynchronous function sends a GET request to the specified
/// URL and parses the response as `LicenseDetails`.
///
/// # Arguments
///
/// * `details_url` - A string slice that holds the URL from which to
///   fetch the license details.
///
/// # Returns
///
/// A `LictoolResult` wrapping a `LicenseDetails` struct on success.
///
/// # Example
///
/// ```
/// let url = "https://example.com/license";
/// let details = fetch_license_details(url).await?;
/// println!("{:?}", details);
/// ```
pub(crate) async fn fetch_license_details(details_url: &str) -> LictoolResult<LicenseDetails> {
    let client = ClientBuilder::new(Client::new())
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: http_cache_dir(),
            },
            options: HttpCacheOptions::default(),
        }))
        .build();
    let res = client
        .get(details_url)
        .send()
        .await?
        .json::<LicenseDetails>()
        .await?;
    Ok(res)
}

/// A struct representing a software license.
///
/// This struct holds essential information about a license, such as
/// its ID, approval status, and URL for more details.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct License {
    // pub reference: String,
    /// A boolean indicating if the license ID is deprecated.
    pub is_deprecated_license_id: bool,
    /// A string containing the URL for detailed information about the
    /// license.
    pub details_url: String,
    // pub reference_number: i64,
    // pub name: String,
    /// The unique identifier of the license.
    #[serde(rename = "licenseId")]
    pub id: String,
    // pub see_also: Vec<String>,
    /// A boolean indicating if the license is approved by the OSI.
    pub is_osi_approved: bool,
    /// An optional boolean indicating if the license is approved by
    /// the FSF.
    pub is_fsf_libre: Option<bool>,
}

impl License {
    /// Fetches detailed information about the license.
    ///
    /// This asynchronous function retrieves the license details from
    /// the URL specified in the `details_url` field of the
    /// `License` struct.
    ///
    /// # Returns
    ///
    /// A `LictoolResult` wrapping a `LicenseDetails` struct on
    /// success.
    ///
    /// # Example
    ///
    /// ```
    /// let license = License { /* initialize fields */ };
    /// let details = license.details().await?;
    /// println!("{:?}", details);
    /// ```
    pub async fn details(&self) -> LictoolResult<LicenseDetails> {
        fetch_license_details(&self.details_url).await
    }

    /// Returns the license ID as a colored string.
    ///
    /// This function formats the license ID as a colored string based
    /// on its deprecation status. Deprecated IDs are displayed in
    /// red, while non-deprecated IDs are displayed in green.
    ///
    /// # Returns
    ///
    /// A `String` containing the colored license ID.
    ///
    /// # Example
    ///
    /// ```
    /// let license = License { /* initialize fields */ };
    /// let colored_id = license.color_id();
    /// println!("{}", colored_id);
    /// ```
    pub fn color_id(&self) -> String {
        if self.is_deprecated_license_id {
            cformat!("<bold, red>{}</>", self.id)
        } else {
            cformat!("<bold, green>{}</>", self.id)
        }
    }
}

impl Display for License {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{fetch_licenses, License, Licenses};
    use crate::spdx::{fetch_license_details, LicenseDetails};

    #[tokio::test]
    async fn test_fetch_licenses() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/licenses/licenses.json")
            .with_body(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/files/licenses.json"
            )))
            .create_async()
            .await;

        let licenses = fetch_licenses(&server.url()).await.unwrap();
        let list = vec![
            License {
                is_deprecated_license_id: false,
                details_url: "https://spdx.org/licenses/BSD-4.3TAHOE.json".to_string(),
                id: "BSD-4.3TAHOE".to_string(),
                is_osi_approved: false,
                is_fsf_libre: None,
            },
            License {
                is_deprecated_license_id: false,
                details_url: "https://spdx.org/licenses/AML-glslang.json".to_string(),
                id: "AML-glslang".to_string(),
                is_osi_approved: false,
                is_fsf_libre: None,
            },
        ];
        let expected = Licenses {
            body: list,
        };
        assert_eq!(licenses, expected);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_fetch_license_details() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/details.json")
            .with_body(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/files/details.json"
            )))
            .create_async()
            .await;
        let details = fetch_license_details(&format!("{}{}", server.url(), "/details.json"))
            .await
            .unwrap();
        let expected = LicenseDetails {
            is_deprecated_license_id: false,
            license_text: "Copyright (c) 2002, NVIDIA Corporation.\n\nNVIDIA \
                           Corporation(\"NVIDIA\") supplies this software to you \
                           in\nconsideration of your agreement to the following terms, and your \
                           use,\ninstallation, modification or redistribution of this NVIDIA \
                           software\nconstitutes acceptance of these terms.  If you do not agree \
                           with these\nterms, please do not use, install, modify or redistribute \
                           this NVIDIA\nsoftware.\n\nIn consideration of your agreement to abide \
                           by the following terms, and\nsubject to these terms, NVIDIA grants you \
                           a personal, non-exclusive\nlicense, under NVIDIA\u{0027}s copyrights \
                           in this original NVIDIA software (the\n\"NVIDIA Software\"), to use, \
                           reproduce, modify and redistribute the\nNVIDIA Software, with or \
                           without modifications, in source and/or binary\nforms; provided that \
                           if you redistribute the NVIDIA Software, you must\nretain the \
                           copyright notice of NVIDIA, this notice and the following\ntext and \
                           disclaimers in all such redistributions of the NVIDIA \
                           Software.\nNeither the name, trademarks, service marks nor logos of \
                           NVIDIA\nCorporation may be used to endorse or promote products derived \
                           from the\nNVIDIA Software without specific prior written permission \
                           from NVIDIA.\nExcept as expressly stated in this notice, no other \
                           rights or licenses\nexpress or implied, are granted by NVIDIA herein, \
                           including but not\nlimited to any patent rights that may be infringed \
                           by your derivative\nworks or by other works in which the NVIDIA \
                           Software may be\nincorporated. No hardware is licensed \
                           hereunder.\n\nTHE NVIDIA SOFTWARE IS BEING PROVIDED ON AN \"AS IS\" \
                           BASIS, WITHOUT\nWARRANTIES OR CONDITIONS OF ANY KIND, EITHER EXPRESS \
                           OR IMPLIED,\nINCLUDING WITHOUT LIMITATION, WARRANTIES OR CONDITIONS OF \
                           TITLE,\nNON-INFRINGEMENT, MERCHANTABILITY, FITNESS FOR A PARTICULAR \
                           PURPOSE, OR\nITS USE AND OPERATION EITHER ALONE OR IN COMBINATION WITH \
                           OTHER\nPRODUCTS.\n\nIN NO EVENT SHALL NVIDIA BE LIABLE FOR ANY \
                           SPECIAL, INDIRECT,\nINCIDENTAL, EXEMPLARY, CONSEQUENTIAL DAMAGES \
                           (INCLUDING, BUT NOT LIMITED\nTO, LOST PROFITS; PROCUREMENT OF \
                           SUBSTITUTE GOODS OR SERVICES; LOSS OF\nUSE, DATA, OR PROFITS; OR \
                           BUSINESS INTERRUPTION) OR ARISING IN ANY WAY\nOUT OF THE USE, \
                           REPRODUCTION, MODIFICATION AND/OR DISTRIBUTION OF THE\nNVIDIA \
                           SOFTWARE, HOWEVER CAUSED AND WHETHER UNDER THEORY OF CONTRACT,\nTORT \
                           (INCLUDING NEGLIGENCE), STRICT LIABILITY OR OTHERWISE, EVEN IF\nNVIDIA \
                           HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.\n"
                .to_string(),
            name: "AML glslang variant License".to_string(),
            license_comments: None,
            license_id: "AML-glslang".to_string(),
            see_also: vec![
                "https://github.com/KhronosGroup/glslang/blob/main/LICENSE.txt#L949".to_string(),
                "https://docs.omniverse.nvidia.com/install-guide/latest/common/licenses.html"
                    .to_string(),
            ],
            is_osi_approved: false,
            is_fsf_libre: None,
            deprecated_version: None,
        };
        assert_eq!(details, expected);
        mock.assert_async().await;
    }
}
