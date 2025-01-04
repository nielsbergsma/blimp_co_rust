use reservation::policy::ReservationPolicy;

pub struct Claims {
    pub scopes: Vec<Scope>
}

#[derive(PartialEq)]
pub enum Scope {
    AgentScope
}

pub fn parse_bearer_token(value: &str) -> Claims {
    let mut scopes = Vec::new();
    if value.contains("agent") {
        scopes.push(Scope::AgentScope);
    }

    Claims {
        scopes
    }
}

pub fn reservation_policy_from_claims(claims: Option<Claims>) -> ReservationPolicy {
    claims
        .map(|claims| {
            if claims.scopes.contains(&Scope::AgentScope) {
                ReservationPolicy::agent()
            }
            else {
                ReservationPolicy::passenger()
            }
        })
        .unwrap_or(ReservationPolicy::passenger())
}

