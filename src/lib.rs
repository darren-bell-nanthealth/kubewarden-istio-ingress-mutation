extern crate wapc_guest as guest;

use guest::prelude::*;

// use k8s_openapi::api::core::v1 as apicore;
use k8s_openapi::api::extensions::v1beta1 as extensions;

extern crate kubewarden_policy_sdk as kubewarden;

use kubewarden::{request::ValidationRequest, validate_settings, accept_request, mutate_request, reject_request};

mod settings;

use settings::Settings;
// use k8s_openapi::serde_json::Value;
use k8s_openapi::api::extensions::v1beta1::Ingress;
use k8s_openapi::api::extensions::v1beta1::IngressTLS;
// use log::{info, trace, warn};


#[no_mangle]
pub extern "C" fn wapc_init() {
    register_function("validate", validate);
    register_function("validate_settings", validate_settings::<Settings>);
}

fn validate(payload: &[u8]) -> CallResult {
    let validation_request: ValidationRequest<Settings> = ValidationRequest::new(payload)?;
    //let ingress = serde_json::from_value::<extensions::Ingress>(validation_req.request.object)?;
    // TODO: REQUIRE VALIDATION FOR CERTAIN ANNOTATIONS
    match serde_json::from_value::<extensions::Ingress>(validation_request.request.object) {
        // NOTE 1
        Ok(ingress) => {
            let mutated_ingress_with_annotations = set_ingress_rewrite_annotations(ingress);
            let mutated_ingress_with_annotations_and_tls = set_ingress_tls_settings(mutated_ingress_with_annotations);

            // NOTE 3
            let mutated_object = serde_json::to_value(mutated_ingress_with_annotations_and_tls).unwrap();
            //mutate_request(&mutated_object)

            reject_request(Some(String::from("Testing the processing of the ingress mutation")), None)

        }
        Err(_) => {
            // We were forwarded a request we cannot unmarshal or
            // understand, just accept it
            reject_request(Some(String::from("Something went wrong parsing the incoming ingress")), None)
            //accept_request()
        }
    }
}

fn set_ingress_tls_settings(mut ingress: Ingress) -> Ingress {
    println!("Ingress Mutation - TLS Settings");
    let mut tls_vec: Vec<IngressTLS> = Vec::new();
    let mut host_vec: Vec<String> = Vec::new();
    let mut ingress_specification = ingress.spec.clone().unwrap_or_default();

    ingress_specification.rules.as_ref().map(|ingress_rules| {
        println!("Ingress Mutation - Annotations: Ingress Rules Found.");
        for rule in ingress_rules.iter() {
            let host = rule.host.clone().unwrap_or_default();
            host_vec.push(host)
        }
    });

    let ingress_class = ingress.metadata.annotations.as_ref().unwrap().get("kubernetes.io/ingress.class");
    let secret_name = match ingress_class.as_ref().unwrap().as_str() {
        "public" => "external-ingress-secret",
        "internal" => "internal-ingress-secret",
        _t => {
            println!("About to panic getting the ingress class - {}", _t );
            panic!("crash and burn");
        }
    };


    match ingress_specification.tls {
        Some(_) => (), //TODO: ADD some log here
        None => {
            let mut tls: IngressTLS = Default::default();
            tls.hosts = Some(host_vec);
            let secret_name = secret_name.to_string();
            tls.secret_name = Some(secret_name);
            tls_vec.push(tls);
            ingress_specification.tls = Some(tls_vec)
        }
    };

    ingress.spec = Some(ingress_specification);

    ingress
}

fn set_ingress_rewrite_annotations(mut ingress: Ingress) -> Ingress {
    let mut new_annotations = ingress.metadata.annotations.clone().unwrap_or_default();
    new_annotations.insert(
        String::from("kubewarden.policy.ingress/inspected"),
        String::from("true"),
    );

    new_annotations.insert(
        String::from("nginx.ingress.kubernetes.io/service-upstream"),
        String::from("true"),
    );

    let default_namespace = "default".to_string();
    let namespace = ingress.metadata.namespace.as_ref().unwrap_or(&default_namespace);
    println!("Ingress Mutation - Annotations: Namespace that is found : {}", namespace);
    ingress.spec.as_ref().map(|ingress_spec| {
        ingress_spec.rules.as_ref().map(|ingress_rules| {
            println!("Ingress Mutation - Annotations: Ingress Rules Found.");
            let service_name = ingress_rules[0].http.as_ref().and_then(|v| {
                v.paths[0].backend.service_name.clone()
            });

            println!("Ingress Mutation - Annotations: Service Name that is found : {}", service_name.clone().unwrap());
            if let Some(service_name) = service_name {
                new_annotations.insert(
                    String::from("nginx.ingress.kubernetes.io/upstream-vhost"),
                    format!("{service_name}.{namespace}.svc.cluster.local", service_name = service_name, namespace = namespace),//String::from("service_name.$namespace.svc.cluster.local"),
                );
            }
        })
    });


    ingress.metadata.annotations = Some(new_annotations);
    ingress
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     use kubewarden_policy_sdk::test::Testcase;
//
//     #[test]
//     fn mutate_ingress_with_annotations_required_for_istio() -> Result<(), ()> {
//         let settings = Settings {
//             secret : String::from("ExternalSecret")
//         };
//
//         let request_file = "test_data/ingress_creation.json";
//         let tc = Testcase {
//             name: String::from("Ingress Creation Istio related Annotations"),
//             fixture_file: String::from(request_file),
//             expected_validation_result: true,
//             settings,
//         };
//
//         let res = tc.eval(validate).unwrap();
//         // NOTE 1
//         assert!(
//             res.mutated_object.is_some(),
//             "Expected accepted object to be mutated",
//         );
//
//         let json_ingess = res.mutated_object.unwrap();
//         println!("{}", json_ingess.as_str());
//         // NOTE 2
//         let final_ingress =
//             serde_json::from_str::<extensions::Ingress>(json_ingess.as_str()).unwrap();
//         let final_annotations = final_ingress.metadata.annotations.unwrap();
//         assert_eq!(
//             final_annotations.get_key_value("kubewarden.policy.ingress/inspected"),
//             Some((
//                 &String::from("kubewarden.policy.ingress/inspected"),
//                 &String::from("true")
//             )),
//         );
//         assert_eq!(
//             final_annotations.get_key_value("nginx.ingress.kubernetes.io/service-upstream"),
//             Some((
//                 &String::from("nginx.ingress.kubernetes.io/service-upstream"),
//                 &String::from("true")
//             )),
//         );
//         assert_eq!(
//             final_annotations.get_key_value("nginx.ingress.kubernetes.io/upstream-vhost"),
//             Some((
//                 &String::from("nginx.ingress.kubernetes.io/upstream-vhost"),
//                 &String::from("service.default.svc.cluster.local")
//             )),
//         );
//
//         Ok(())
//     }
//
//     #[test]
//     fn mutate_complex_ingress_with_annotations_required_for_istio() -> Result<(), ()> {
//         let settings = Settings {
//             secret : String::from("ExternalSecret")
//         };
//
//         let request_file = "test_data/1_ingress_multiple_rules_without_rewrite_annotation.json";
//         let tc = Testcase {
//             name: String::from("Ingress Creation Istio related Annotations"),
//             fixture_file: String::from(request_file),
//             expected_validation_result: true,
//             settings,
//         };
//
//         let res = tc.eval(validate).unwrap();
//         // NOTE 1
//         assert!(
//             res.mutated_object.is_some(),
//             "Expected accepted object to be mutated",
//         );
//
//         let json_ingess = res.mutated_object.unwrap();
//         println!("{}", json_ingess.as_str());
//         // NOTE 2
//         let final_ingress =
//             serde_json::from_str::<extensions::Ingress>(json_ingess.as_str()).unwrap();
//         let final_annotations = final_ingress.metadata.annotations.unwrap();
//         assert_eq!(
//             final_annotations.get_key_value("kubewarden.policy.ingress/inspected"),
//             Some((
//                 &String::from("kubewarden.policy.ingress/inspected"),
//                 &String::from("true")
//             )),
//         );
//         assert_eq!(
//             final_annotations.get_key_value("nginx.ingress.kubernetes.io/service-upstream"),
//             Some((
//                 &String::from("nginx.ingress.kubernetes.io/service-upstream"),
//                 &String::from("true")
//             )),
//         );
//         assert_eq!(
//             final_annotations.get_key_value("nginx.ingress.kubernetes.io/upstream-vhost"),
//             Some((
//                 &String::from("nginx.ingress.kubernetes.io/upstream-vhost"),
//                 &String::from("kelp-kelp-svc.default.svc.cluster.local")
//             )),
//         );
//
//         Ok(())
//     }
//
//
//     #[test]
//     fn mutate_ingress_with_tls_required_for_tls_termination() -> Result<(), ()> {
//         let settings = Settings {
//             secret : String::from("ExternalSecret")
//         };
//
//         let request_file = "test_data/ingress_creation.json";
//         let tc = Testcase {
//             name: String::from("Ingress Creation add annotations"),
//             fixture_file: String::from(request_file),
//             expected_validation_result: true,
//             settings,
//         };
//
//         let res = tc.eval(validate).unwrap();
//         // NOTE 1
//         assert!(
//             res.mutated_object.is_some(),
//             "Expected accepted object to be mutated",
//         );
//
//         let json_ingess = res.mutated_object.unwrap();
//         println!("{}", json_ingess.as_str());
//
//         let final_ingress =
//             serde_json::from_str::<extensions::Ingress>(json_ingess.as_str()).unwrap();
//         let tls_vec = final_ingress.spec.unwrap().tls.unwrap();
//
//         assert_eq!(tls_vec.len(), 1, "We are only expecting 1 tls entry but got {} ", tls_vec.len());
//
//         assert_eq!(
//             tls_vec[0].secret_name,
//             Some(String::from("internal-ingress-secret"))
//         );
//         assert_eq!(
//             tls_vec[0].hosts.as_ref().unwrap()[0],
//             "some-host.com",
//         );
//
//         Ok(())
//     }
//
// }
