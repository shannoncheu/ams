# Published-crate license clarifications

Some published crates omit a standalone license file. These explicit,
version-specific choices supplement their published SPDX declarations; they do
not relicense the application or alter the dependency source. The complete
unchanged published source, original Cargo.toml attribution and source headers
are included in THIRD_PARTY_SOURCES.zip for these packages.

| Package | Version | Distribution choice | Published evidence |
| --- | --- | --- | --- |
| dwrote | 0.11.5 | MPL-2.0 | Cargo.toml and MPL Exhibit A notices in src/*.rs |
| pathfinder_geometry | 0.5.1 | Apache-2.0 | Cargo.toml declares MIT/Apache-2.0; src/vector.rs explicitly offers Apache-2.0 or MIT and credits The Pathfinder Project Developers |
| seahash | 4.1.0 | MIT | Published Cargo.toml license and original author attribution; all original source is supplied |
| simd_helpers | 0.1.0 | MIT | Published Cargo.toml license and original author attribution; all original source is supplied |
| zune-inflate | 0.2.54 | Apache-2.0 | Published Cargo.toml offers MIT/Apache-2.0/Zlib; source headers repeat that grant and retain original copyright |

Canonical Apache-2.0 and MPL-2.0 license texts are supplied unchanged. MIT-TERMS.txt
contains the standard MIT permission/conditions/disclaimer; attribution comes
from each package's original published manifests and source, rather than adding
an invented copyright holder or year. Existing source copyright notices remain
in the supplied source archive. All other missing license files must be resolved
from the published VCS commit or packaging stops.
