use crate::error::Error;

use crate::parser::asn::structs::types::{
    Asn1BuiltinType, Asn1Type, Asn1TypeKind, Asn1TypeReference,
};

use crate::resolver::{
    asn::structs::types::{
        base::{
            Asn1ResolvedBitString, Asn1ResolvedBoolean, Asn1ResolvedCharacterString,
            Asn1ResolvedEnumerated, Asn1ResolvedInteger, Asn1ResolvedObjectIdentifier,
            Asn1ResolvedOctetString, ResolvedBaseType,
        },
        constraints::Asn1ConstraintValueSet,
    },
    Resolver,
};

impl Asn1Type {
    // Returns the Integer ValueSet for a given Type.
    //
    // If the type is a `Base` type, it should be INTEGER or it's an Error. If the `Type` is a
    // Referenced Type, it should be possible to 'resolve' the Reference to a proper
    // `Asn1ConstraintValueSet` else it's an error, we'll try to 'recursively' 'resolve' till we
    // get to a `Base` Type.
    pub(crate) fn get_integer_valueset_from_constraint(
        &self,
        resolver: &Resolver,
    ) -> Result<Asn1ConstraintValueSet, Error> {
        let kind = &self.kind;
        match kind {
            Asn1TypeKind::Builtin(Asn1BuiltinType::Integer(..)) => {
                let constraint = &self.constraints.as_ref().unwrap()[0];
                constraint.get_integer_valueset(resolver)
            }
            Asn1TypeKind::Reference(Asn1TypeReference::Reference(ref _r)) => {
                Err(constraint_error!("Not Implemented!"))
            }
            _ => Err(constraint_error!(
                "The Type '{:#?}' is not of a BuiltIn Or a Referenced Kind!",
                self,
            )),
        }
    }
}

pub(crate) fn resolve_base_type(
    ty: &Asn1Type,
    resolver: &Resolver,
) -> Result<ResolvedBaseType, Error> {
    if let Asn1TypeKind::Builtin(ref kind) = ty.kind {
        match kind {
            Asn1BuiltinType::Integer(ref _i) => {
                let mut resolved = Asn1ResolvedInteger::default();
                resolve_integer(&mut resolved, ty, resolver)?;
                Ok(ResolvedBaseType::Integer(resolved))
            }
            Asn1BuiltinType::Enumerated(ref _i) => {
                let resolved = ResolvedBaseType::Enum(Asn1ResolvedEnumerated::default());
                Ok(resolved)
            }
            Asn1BuiltinType::BitString(ref _i) => {
                let resolved = ResolvedBaseType::BitString(Asn1ResolvedBitString::default());
                Ok(resolved)
            }
            Asn1BuiltinType::Boolean => {
                let resolved = ResolvedBaseType::Boolean(Asn1ResolvedBoolean::default());
                Ok(resolved)
            }
            Asn1BuiltinType::OctetString => {
                let resolved = ResolvedBaseType::OctetString(Asn1ResolvedOctetString::default());
                Ok(resolved)
            }
            Asn1BuiltinType::CharacterString => {
                let resolved =
                    ResolvedBaseType::CharacterString(Asn1ResolvedCharacterString::default());
                Ok(resolved)
            }
            Asn1BuiltinType::ObjectIdentifier => {
                let resolved =
                    ResolvedBaseType::ObjectIdentifier(Asn1ResolvedObjectIdentifier::default());
                Ok(resolved)
            }
            _ => Err(resolve_error!(
                "parse_base_type: Not Implemented! {:#?}",
                ty
            )),
        }
    } else {
        Err(resolve_error!("Expected Base Type. Found '{:#?}'", ty))
    }
}

fn resolve_integer(
    base: &mut Asn1ResolvedInteger,
    ty: &Asn1Type,
    resolver: &Resolver,
) -> Result<(), Error> {
    // No Constraints
    eprintln!("1... {:#?}", ty);
    if ty.constraints.is_none() {
        return Ok(());
    } else {
        let constraints = ty.constraints.as_ref().unwrap();
        if constraints.is_empty() {
            return Ok(());
        }
    }

    // Get the Values that are expected
    let value_set = ty.get_integer_valueset_from_constraint(resolver)?;
    if let Some(x) = value_set.root_values.min() {
        if x < 0 {
            base.signed = true
        }
    }

    let _ = base.resolved_constraints.replace(value_set);
    Ok(())
}
