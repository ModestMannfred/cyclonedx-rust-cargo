/*
 * This file is part of CycloneDX Rust Cargo.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{
    models::{self, bom::SpecVersion},
    utilities::convert_optional,
    xml::{
        expected_namespace_or_error, optional_attribute, read_lax_validation_tag,
        to_xml_read_error, to_xml_write_error, unexpected_element_error, FromXml, FromXmlDocument,
        FromXmlType,
    },
};
use crate::{
    specs::v1_4::{
        component::Components, composition::Compositions, dependency::Dependencies,
        external_reference::ExternalReferences, metadata::Metadata, property::Properties,
        service::Services, signature::Signature, vulnerability::Vulnerabilities,
    },
    xml::ToXml,
};
use serde::{Deserialize, Serialize};
use xml::{reader, writer::XmlEvent};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Bom {
    bom_format: BomFormat,
    spec_version: SpecVersion,
    version: Option<u32>,
    serial_number: Option<UrnUuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    components: Option<Components>,
    #[serde(skip_serializing_if = "Option::is_none")]
    services: Option<Services>,
    #[serde(skip_serializing_if = "Option::is_none")]
    external_references: Option<ExternalReferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dependencies: Option<Dependencies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compositions: Option<Compositions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Properties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vulnerabilities: Option<Vulnerabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<Signature>,
}

impl From<models::bom::Bom> for Bom {
    fn from(other: models::bom::Bom) -> Self {
        Self {
            bom_format: BomFormat::CycloneDX,
            spec_version: SpecVersion::V1_4,
            version: Some(other.version),
            serial_number: convert_optional(other.serial_number),
            metadata: convert_optional(other.metadata),
            components: convert_optional(other.components),
            services: convert_optional(other.services),
            external_references: convert_optional(other.external_references),
            dependencies: convert_optional(other.dependencies),
            compositions: convert_optional(other.compositions),
            properties: convert_optional(other.properties),
            vulnerabilities: convert_optional(other.vulnerabilities),
            signature: convert_optional(other.signature),
        }
    }
}

impl From<Bom> for models::bom::Bom {
    fn from(other: Bom) -> Self {
        Self {
            version: other.version.unwrap_or(1),
            serial_number: convert_optional(other.serial_number),
            metadata: convert_optional(other.metadata),
            components: convert_optional(other.components),
            services: convert_optional(other.services),
            external_references: convert_optional(other.external_references),
            dependencies: convert_optional(other.dependencies),
            compositions: convert_optional(other.compositions),
            properties: convert_optional(other.properties),
            vulnerabilities: convert_optional(other.vulnerabilities),
            signature: convert_optional(other.signature),
        }
    }
}

const BOM_TAG: &str = "bom";
const SERIAL_NUMBER_ATTR: &str = "serialNumber";
const VERSION_ATTR: &str = "version";

impl ToXml for Bom {
    fn write_xml_element<W: std::io::Write>(
        &self,
        writer: &mut xml::EventWriter<W>,
    ) -> Result<(), crate::errors::XmlWriteError> {
        let version = self.version.map(|v| format!("{}", v));
        let mut bom_start_element =
            XmlEvent::start_element(BOM_TAG).default_ns("http://cyclonedx.org/schema/bom/1.4");

        if let Some(serial_number) = &self.serial_number {
            bom_start_element = bom_start_element.attr(SERIAL_NUMBER_ATTR, &serial_number.0);
        }

        if let Some(version) = &version {
            bom_start_element = bom_start_element.attr(VERSION_ATTR, version);
        }

        writer
            .write(bom_start_element)
            .map_err(to_xml_write_error(BOM_TAG))?;

        if let Some(metadata) = &self.metadata {
            metadata.write_xml_element(writer)?;
        }

        if let Some(components) = &self.components {
            components.write_xml_element(writer)?;
        }

        if let Some(services) = &self.services {
            services.write_xml_element(writer)?;
        }

        if let Some(external_references) = &self.external_references {
            external_references.write_xml_element(writer)?;
        }

        if let Some(dependencies) = &self.dependencies {
            dependencies.write_xml_element(writer)?;
        }

        if let Some(compositions) = &self.compositions {
            compositions.write_xml_element(writer)?;
        }

        if let Some(properties) = &self.properties {
            properties.write_xml_element(writer)?;
        }

        if let Some(vulnerabilities) = &self.vulnerabilities {
            vulnerabilities.write_xml_element(writer)?;
        }

        writer
            .write(XmlEvent::end_element())
            .map_err(to_xml_write_error(BOM_TAG))?;

        Ok(())
    }
}

const METADATA_TAG: &str = "metadata";
const COMPONENTS_TAG: &str = "components";
const SERVICES_TAG: &str = "services";
const EXTERNAL_REFERENCES_TAG: &str = "externalReferences";
const DEPENDENCIES_TAG: &str = "dependencies";
const COMPOSITIONS_TAG: &str = "compositions";
const PROPERTIES_TAG: &str = "properties";
const VULNERABILITIES_TAG: &str = "vulnerabilities";
const SIGNATURE_TAG: &str = "signature";

impl FromXmlDocument for Bom {
    fn read_xml_document<R: std::io::Read>(
        event_reader: &mut xml::EventReader<R>,
    ) -> Result<Self, crate::errors::XmlReadError>
    where
        Self: Sized,
    {
        event_reader
            .next()
            .map_err(to_xml_read_error(BOM_TAG))
            .and_then(|event| match event {
                reader::XmlEvent::StartDocument { .. } => Ok(()),
                unexpected => Err(unexpected_element_error(BOM_TAG, unexpected)),
            })?;

        let (version, serial_number) = event_reader
            .next()
            .map_err(to_xml_read_error(BOM_TAG))
            .and_then(|event| match event {
                reader::XmlEvent::StartElement {
                    name,
                    attributes,
                    namespace,
                } if name.local_name == BOM_TAG => {
                    expected_namespace_or_error("1.4", &namespace)?;
                    let version =
                        if let Some(version) = optional_attribute(&attributes, VERSION_ATTR) {
                            let version = u32::from_xml_value(VERSION_ATTR, version)?;
                            Some(version)
                        } else {
                            None
                        };
                    let serial_number =
                        optional_attribute(&attributes, SERIAL_NUMBER_ATTR).map(UrnUuid);
                    Ok((version, serial_number))
                }
                unexpected => Err(unexpected_element_error(BOM_TAG, unexpected)),
            })?;

        let mut metadata: Option<Metadata> = None;
        let mut components: Option<Components> = None;
        let mut services: Option<Services> = None;
        let mut external_references: Option<ExternalReferences> = None;
        let mut dependencies: Option<Dependencies> = None;
        let mut compositions: Option<Compositions> = None;
        let mut properties: Option<Properties> = None;
        let mut vulnerabilities: Option<Vulnerabilities> = None;
        let mut signature: Option<Signature> = None;

        let mut got_end_tag = false;
        while !got_end_tag {
            let next_element = event_reader.next().map_err(to_xml_read_error(BOM_TAG))?;
            match next_element {
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == METADATA_TAG => {
                    metadata = Some(Metadata::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMPONENTS_TAG => {
                    components = Some(Components::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SERVICES_TAG => {
                    services = Some(Services::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == EXTERNAL_REFERENCES_TAG => {
                    external_references = Some(ExternalReferences::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == DEPENDENCIES_TAG => {
                    dependencies = Some(Dependencies::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == COMPOSITIONS_TAG => {
                    compositions = Some(Compositions::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == PROPERTIES_TAG => {
                    properties = Some(Properties::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == VULNERABILITIES_TAG => {
                    vulnerabilities = Some(Vulnerabilities::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }
                reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == SIGNATURE_TAG => {
                    signature = Some(Signature::read_xml_element(
                        event_reader,
                        &name,
                        &attributes,
                    )?)
                }

                // lax validation of any elements from a different schema
                reader::XmlEvent::StartElement { name, .. } => {
                    read_lax_validation_tag(event_reader, &name)?
                }
                reader::XmlEvent::EndElement { name } if name.local_name == BOM_TAG => {
                    got_end_tag = true;
                }
                unexpected => return Err(unexpected_element_error(BOM_TAG, unexpected)),
            }
        }

        event_reader
            .next()
            .map_err(to_xml_read_error(BOM_TAG))
            .and_then(|event| match event {
                reader::XmlEvent::EndDocument => Ok(()),
                unexpected => Err(unexpected_element_error(BOM_TAG, unexpected)),
            })?;
        Ok(Self {
            bom_format: BomFormat::CycloneDX,
            spec_version: SpecVersion::V1_4,
            version,
            serial_number,
            metadata,
            components,
            services,
            external_references,
            dependencies,
            compositions,
            properties,
            vulnerabilities,
            signature,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
enum BomFormat {
    CycloneDX,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct UrnUuid(String);

impl From<models::bom::UrnUuid> for UrnUuid {
    fn from(other: models::bom::UrnUuid) -> Self {
        Self(other.0)
    }
}

impl From<UrnUuid> for models::bom::UrnUuid {
    fn from(other: UrnUuid) -> Self {
        Self(other.0)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use crate::specs::v1_4::vulnerability::test::{
        corresponding_vulnerabilities, example_vulnerabilities,
    };
    use crate::{
        specs::v1_4::{
            component::test::{corresponding_components, example_components},
            composition::test::{corresponding_compositions, example_compositions},
            dependency::test::{corresponding_dependencies, example_dependencies},
            external_reference::test::{
                corresponding_external_references, example_external_references,
            },
            metadata::test::{corresponding_metadata, example_metadata},
            property::test::{corresponding_properties, example_properties},
            service::test::{corresponding_services, example_services},
            signature::test::{corresponding_signature, example_signature},
        },
        xml::test::{read_document_from_string, write_element_to_string},
    };

    use super::*;

    pub(crate) fn minimal_bom_example() -> Bom {
        Bom {
            bom_format: BomFormat::CycloneDX,
            spec_version: SpecVersion::V1_4,
            version: Some(1),
            serial_number: Some(UrnUuid("fake-uuid".to_string())),
            metadata: None,
            components: None,
            services: None,
            external_references: None,
            dependencies: None,
            compositions: None,
            properties: None,
            vulnerabilities: None,
            signature: None,
        }
    }

    pub(crate) fn full_bom_example() -> Bom {
        Bom {
            bom_format: BomFormat::CycloneDX,
            spec_version: SpecVersion::V1_4,
            version: Some(1),
            serial_number: Some(UrnUuid("fake-uuid".to_string())),
            metadata: Some(example_metadata()),
            components: Some(example_components()),
            services: Some(example_services()),
            external_references: Some(example_external_references()),
            dependencies: Some(example_dependencies()),
            compositions: Some(example_compositions()),
            properties: Some(example_properties()),
            vulnerabilities: Some(example_vulnerabilities()),
            signature: Some(example_signature()),
        }
    }

    pub(crate) fn corresponding_internal_model() -> models::bom::Bom {
        models::bom::Bom {
            version: 1,
            serial_number: Some(models::bom::UrnUuid("fake-uuid".to_string())),
            metadata: Some(corresponding_metadata()),
            components: Some(corresponding_components()),
            services: Some(corresponding_services()),
            external_references: Some(corresponding_external_references()),
            dependencies: Some(corresponding_dependencies()),
            compositions: Some(corresponding_compositions()),
            properties: Some(corresponding_properties()),
            vulnerabilities: Some(corresponding_vulnerabilities()),
            signature: Some(corresponding_signature()),
        }
    }

    #[test]
    fn it_should_serialize_to_json() {
        insta::assert_json_snapshot!(minimal_bom_example());
    }

    #[test]
    fn it_should_serialize_to_xml() {
        let xml_output = write_element_to_string(minimal_bom_example());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_should_serialize_a_complex_example_to_json() {
        let actual = full_bom_example();

        insta::assert_json_snapshot!(actual);
    }

    #[test]
    fn it_should_serialize_a_complex_example_to_xml() {
        let xml_output = write_element_to_string(full_bom_example());
        insta::assert_snapshot!(xml_output);
    }

    #[test]
    fn it_can_convert_to_the_internal_model() {
        let spec = full_bom_example();
        let model: models::bom::Bom = spec.into();
        assert_eq!(model, corresponding_internal_model());
    }

    #[test]
    fn it_can_convert_from_the_internal_model() {
        let model = corresponding_internal_model();
        let spec: Bom = model.into();
        assert_eq!(spec, full_bom_example());
    }

    #[test]
    fn it_should_deserialize_from_xml() {
        let input = r#"
<?xml version="1.0" encoding="utf-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.4" serialNumber="fake-uuid" version="1" />
"#
        .trim_start();
        let actual: Bom = read_document_from_string(input);
        let expected = minimal_bom_example();
        assert_eq!(actual, expected);
    }

    #[test]
    fn it_should_deserialize_a_complex_example_from_xml() {
        let input = r#"
<?xml version="1.0" encoding="utf-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.4" xmlns:example="https://example.com" serialNumber="fake-uuid" version="1">
  <metadata>
    <timestamp>timestamp</timestamp>
    <tools>
      <tool>
        <vendor>vendor</vendor>
        <name>name</name>
        <version>version</version>
        <hashes>
          <hash alg="algorithm">hash value</hash>
        </hashes>
      </tool>
    </tools>
    <authors>
      <author>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </author>
    </authors>
    <component type="component type" mime-type="mime type" bom-ref="bom ref">
      <supplier>
        <name>name</name>
        <url>url</url>
        <contact>
          <name>name</name>
          <email>email</email>
          <phone>phone</phone>
        </contact>
      </supplier>
      <author>author</author>
      <publisher>publisher</publisher>
      <group>group</group>
      <name>name</name>
      <version>version</version>
      <description>description</description>
      <scope>scope</scope>
      <hashes>
        <hash alg="algorithm">hash value</hash>
      </hashes>
      <licenses>
        <expression>expression</expression>
      </licenses>
      <copyright>copyright</copyright>
      <cpe>cpe</cpe>
      <purl>purl</purl>
      <swid tagId="tag id" name="name" version="version" tagVersion="1" patch="true">
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
      </swid>
      <modified>true</modified>
      <pedigree>
        <ancestors />
        <descendants />
        <variants />
        <commits>
          <commit>
            <uid>uid</uid>
            <url>url</url>
            <author>
              <timestamp>timestamp</timestamp>
              <name>name</name>
              <email>email</email>
            </author>
            <committer>
              <timestamp>timestamp</timestamp>
              <name>name</name>
              <email>email</email>
            </committer>
            <message>message</message>
          </commit>
        </commits>
        <patches>
          <patch type="patch type">
            <diff>
              <text content-type="content type" encoding="encoding">content</text>
              <url>url</url>
            </diff>
            <resolves>
              <issue type="issue type">
                <id>id</id>
                <name>name</name>
                <description>description</description>
                <source>
                  <name>name</name>
                  <url>url</url>
                </source>
                <references>
                  <url>reference</url>
                </references>
              </issue>
            </resolves>
          </patch>
        </patches>
        <notes>notes</notes>
      </pedigree>
      <externalReferences>
        <reference type="external reference type">
          <url>url</url>
          <comment>comment</comment>
          <hashes>
            <hash alg="algorithm">hash value</hash>
          </hashes>
        </reference>
      </externalReferences>
      <properties>
        <property name="name">value</property>
      </properties>
      <components />
      <evidence>
        <licenses>
          <expression>expression</expression>
        </licenses>
        <copyright>
          <text><![CDATA[copyright]]></text>
        </copyright>
      </evidence>
      <signature>
        <algorithm>HS512</algorithm>
        <value>1234567890</value>
      </signature>
    </component>
    <manufacture>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </manufacture>
    <supplier>
      <name>name</name>
      <url>url</url>
      <contact>
        <name>name</name>
        <email>email</email>
        <phone>phone</phone>
      </contact>
    </supplier>
    <licenses>
      <expression>expression</expression>
    </licenses>
    <properties>
      <property name="name">value</property>
    </properties>
  </metadata>
  <components>
    <component type="component type" mime-type="mime type" bom-ref="bom ref">
      <supplier>
        <name>name</name>
        <url>url</url>
        <contact>
          <name>name</name>
          <email>email</email>
          <phone>phone</phone>
        </contact>
      </supplier>
      <author>author</author>
      <publisher>publisher</publisher>
      <group>group</group>
      <name>name</name>
      <version>version</version>
      <description>description</description>
      <scope>scope</scope>
      <hashes>
        <hash alg="algorithm">hash value</hash>
      </hashes>
      <licenses>
        <expression>expression</expression>
      </licenses>
      <copyright>copyright</copyright>
      <cpe>cpe</cpe>
      <purl>purl</purl>
      <swid tagId="tag id" name="name" version="version" tagVersion="1" patch="true">
        <text content-type="content type" encoding="encoding">content</text>
        <url>url</url>
      </swid>
      <modified>true</modified>
      <pedigree>
        <ancestors />
        <descendants />
        <variants />
        <commits>
          <commit>
            <uid>uid</uid>
            <url>url</url>
            <author>
              <timestamp>timestamp</timestamp>
              <name>name</name>
              <email>email</email>
            </author>
            <committer>
              <timestamp>timestamp</timestamp>
              <name>name</name>
              <email>email</email>
            </committer>
            <message>message</message>
          </commit>
        </commits>
        <patches>
          <patch type="patch type">
            <diff>
              <text content-type="content type" encoding="encoding">content</text>
              <url>url</url>
            </diff>
            <resolves>
              <issue type="issue type">
                <id>id</id>
                <name>name</name>
                <description>description</description>
                <source>
                  <name>name</name>
                  <url>url</url>
                </source>
                <references>
                  <url>reference</url>
                </references>
              </issue>
            </resolves>
          </patch>
        </patches>
        <notes>notes</notes>
      </pedigree>
      <externalReferences>
        <reference type="external reference type">
          <url>url</url>
          <comment>comment</comment>
          <hashes>
            <hash alg="algorithm">hash value</hash>
          </hashes>
        </reference>
      </externalReferences>
      <properties>
        <property name="name">value</property>
      </properties>
      <components />
      <evidence>
        <licenses>
          <expression>expression</expression>
        </licenses>
        <copyright>
          <text><![CDATA[copyright]]></text>
        </copyright>
      </evidence>
      <signature>
        <algorithm>HS512</algorithm>
        <value>1234567890</value>
      </signature>
    </component>
  </components>
  <services>
    <service bom-ref="bom-ref">
      <provider>
        <name>name</name>
        <url>url</url>
        <contact>
          <name>name</name>
          <email>email</email>
          <phone>phone</phone>
        </contact>
      </provider>
      <group>group</group>
      <name>name</name>
      <version>version</version>
      <description>description</description>
      <endpoints>
        <endpoint>endpoint</endpoint>
      </endpoints>
      <authenticated>true</authenticated>
      <x-trust-boundary>true</x-trust-boundary>
      <data>
        <classification flow="flow">classification</classification>
      </data>
      <licenses>
        <expression>expression</expression>
      </licenses>
      <externalReferences>
        <reference type="external reference type">
          <url>url</url>
          <comment>comment</comment>
          <hashes>
            <hash alg="algorithm">hash value</hash>
          </hashes>
        </reference>
      </externalReferences>
      <properties>
        <property name="name">value</property>
      </properties>
      <services />
      <signature>
        <algorithm>HS512</algorithm>
        <value>1234567890</value>
      </signature>
    </service>
  </services>
  <externalReferences>
    <reference type="external reference type">
      <url>url</url>
      <comment>comment</comment>
      <hashes>
        <hash alg="algorithm">hash value</hash>
      </hashes>
    </reference>
  </externalReferences>
  <dependencies>
    <dependency ref="ref">
      <dependency ref="depends on" />
    </dependency>
  </dependencies>
  <compositions>
    <composition>
      <aggregate>aggregate</aggregate>
      <assemblies>
        <assembly ref="assembly" />
      </assemblies>
      <dependencies>
        <dependency ref="dependency" />
      </dependencies>
      <signature>
        <algorithm>HS512</algorithm>
        <value>1234567890</value>
      </signature>
    </composition>
  </compositions>
  <properties>
    <property name="name">value</property>
  </properties>
  <vulnerabilities>
    <vulnerability bom-ref="bom-ref">
      <id>id</id>
      <source>
        <name>name</name>
        <url>url</url>
      </source>
      <references>
        <reference>
          <id>id</id>
          <source>
            <name>name</name>
            <url>url</url>
          </source>
        </reference>
      </references>
      <ratings>
        <rating>
          <source>
            <name>name</name>
            <url>url</url>
          </source>
          <score>9.8</score>
          <severity>info</severity>
          <method>CVSSv3</method>
          <vector>vector</vector>
          <justification>justification</justification>
        </rating>
      </ratings>
      <cwes>
        <cwe>1</cwe>
        <cwe>2</cwe>
        <cwe>3</cwe>
      </cwes>
      <description>description</description>
      <detail>detail</detail>
      <recommendation>recommendation</recommendation>
      <advisories>
        <advisory>
          <title>title</title>
          <url>url</url>
        </advisory>
      </advisories>
      <created>created</created>
      <published>published</published>
      <updated>updated</updated>
      <credits>
        <organizations>
          <organization>
            <name>name</name>
            <url>url</url>
            <contact>
              <name>name</name>
              <email>email</email>
              <phone>phone</phone>
            </contact>
          </organization>
        </organizations>
        <individuals>
          <individual>
            <name>name</name>
            <email>email</email>
            <phone>phone</phone>
          </individual>
        </individuals>
      </credits>
      <tools>
        <tool>
          <vendor>vendor</vendor>
          <name>name</name>
          <version>version</version>
          <hashes>
            <hash alg="algorithm">hash value</hash>
          </hashes>
        </tool>
      </tools>
      <analysis>
        <state>not_affected</state>
        <justification>code_not_reachable</justification>
        <responses>
          <response>update</response>
        </responses>
        <detail>detail</detail>
      </analysis>
      <affects>
        <target>
          <ref>ref</ref>
          <versions>
            <version>
              <version>5.0.0</version>
              <status>unaffected</status>
            </version>
            <version>
              <range>vers:npm/1.2.3|>=2.0.0|&lt;5.0.0</range>
              <status>affected</status>
            </version>
          </versions>
        </target>
      </affects>
      <properties>
        <property name="name">value</property>
      </properties>
    </vulnerability>
  </vulnerabilities>
  <signature>
    <algorithm>HS512</algorithm>
    <value>1234567890</value>
  </signature>
  <example:laxValidation>
    <example:innerElement id="test" />
  </example:laxValidation>
</bom>
"#.trim_start();
        let actual: Bom = read_document_from_string(input);
        let expected = full_bom_example();
        assert_eq!(actual, expected);
    }
}
