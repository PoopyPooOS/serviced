use crate::types::Service;
use std::collections::{BTreeMap, VecDeque};

pub fn sort_services(services: &[Service]) -> Result<Vec<Service>, String> {
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new(); // service id -> dependencies
    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new(); // service id -> number of services depending on it
    let mut all_services: BTreeMap<String, Service> = BTreeMap::new();

    // Build the graph and in-degree map
    for service in services {
        all_services.insert(service.id.clone(), service.clone());
        in_degree.entry(service.id.clone()).or_insert(0);

        for dep in &service.dependencies {
            graph.entry(dep.clone()).or_default().push(service.id.clone());
            *in_degree.entry(service.id.clone()).or_insert(0) += 1;
        }
    }

    // Queue for services with no dependencies (in-degree 0)
    let mut queue: VecDeque<Service> = VecDeque::new();

    for service in all_services.values() {
        if in_degree[&service.id] == 0 {
            queue.push_back(service.clone());
        }
    }

    // Perform topological sorting
    let mut sorted_services = Vec::new();

    while let Some(service) = queue.pop_front() {
        sorted_services.push(service.clone());

        if let Some(deps) = graph.get(&service.id) {
            for dep_id in deps {
                // Decrease in-degree of each dependent service
                if let Some(dependent_service) = all_services.get_mut(dep_id) {
                    *in_degree.get_mut(dep_id).unwrap() -= 1;

                    // If in-degree is 0, it can now be processed
                    if in_degree[dep_id] == 0 {
                        queue.push_back(dependent_service.clone());
                    }
                }
            }
        }
    }

    // If we processed all services, return the sorted list
    if sorted_services.len() == services.len() {
        Ok(sorted_services)
    } else {
        // There was a cycle, return an error
        Err("Cycle detected in dependencies".to_string())
    }
}
