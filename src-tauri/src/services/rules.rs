use std::collections::HashMap;
use crate::models::Rule;
use crate::utils::Platform;

pub fn evaluate_rules(rules: &[Rule], features: &HashMap<String, bool>) -> bool {
    let platform = Platform::current();
    
    // 如果没有 rules，默认允许
    if rules.is_empty() {
        return true;
    }
    
    // 检查是否有匹配的 allow 规则
    for rule in rules {
        let os_match = match &rule.os {
            Some(os) => {
                let name_ok = os.name.as_ref().map(|n| n == platform.name()).unwrap_or(true);
                let arch_ok = os.arch.as_ref().map(|a| a == platform.arch()).unwrap_or(true);
                name_ok && arch_ok
            },
            None => true,
        };
        
        let feat_match = rule.features.as_ref()
            .map(|f| f.iter().all(|(k, v)| features.get(k) == Some(v)))
            .unwrap_or(true);
        
        if os_match && feat_match {
            return rule.action == "allow";
        }
    }
    
    // 如果没有匹配的规则，默认不允许
    false
}