// Copyright (C) 2022-present The NetGauze Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Deserializer for BGP Update message

use crate::{
    serde::deserializer::{
        ipv4_network_from_wire, path_attribute::PathAttributeParsingError, BGPMessageParsingError,
        Ipv4PrefixParsingError, LocatedBGPMessageParsingError,
    },
    update::{NetworkLayerReachabilityInformation, WithdrawRoute},
    BGPUpdateMessage,
};
use ipnet::Ipv4Net;
use netgauze_parse_utils::{
    parse_till_empty_into_located, parse_till_empty_into_with_one_input_located, IntoLocatedError,
    LocatedParsingError, ReadablePDU, ReadablePDUWithOneInput, Span,
};
use nom::{
    error::{ErrorKind, FromExternalError},
    number::complete::be_u16,
    IResult,
};

/// BGP Open Message Parsing errors
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BGPUpdateMessageParsingError {
    /// Errors triggered by the nom parser, see [nom::error::ErrorKind] for
    /// additional information.
    NomError(ErrorKind),
    WithdrawRouteError(WithdrawRouteParsingError),
    PathAttributeError(PathAttributeParsingError),
    NetworkLayerReachabilityInformationError(NetworkLayerReachabilityInformationParsingError),
}

/// BGP Open Message Parsing errors  with the input location of where it
/// occurred in the input byte stream being parsed
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LocatedBGPUpdateMessageParsingError<'a> {
    span: Span<'a>,
    error: BGPUpdateMessageParsingError,
}

impl<'a> LocatedBGPUpdateMessageParsingError<'a> {
    pub const fn new(span: Span<'a>, error: BGPUpdateMessageParsingError) -> Self {
        Self { span, error }
    }
}

impl<'a> LocatedParsingError<'a, BGPUpdateMessageParsingError>
    for LocatedBGPUpdateMessageParsingError<'a>
{
    fn span(&self) -> &Span<'a> {
        &self.span
    }

    fn error(&self) -> &BGPUpdateMessageParsingError {
        &self.error
    }
}

impl<'a> IntoLocatedError<'a, BGPMessageParsingError, LocatedBGPMessageParsingError<'a>>
    for LocatedBGPUpdateMessageParsingError<'a>
{
    fn into_located(self) -> LocatedBGPMessageParsingError<'a> {
        LocatedBGPMessageParsingError::new(
            self.span,
            BGPMessageParsingError::BGPUpdateMessageParsingError(self.error),
        )
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for LocatedBGPUpdateMessageParsingError<'a> {
    fn from_error_kind(input: Span<'a>, kind: ErrorKind) -> Self {
        LocatedBGPUpdateMessageParsingError::new(
            input,
            BGPUpdateMessageParsingError::NomError(kind),
        )
    }

    fn append(_input: Span<'a>, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'a> FromExternalError<Span<'a>, BGPUpdateMessageParsingError>
    for LocatedBGPUpdateMessageParsingError<'a>
{
    fn from_external_error(
        input: Span<'a>,
        _kind: ErrorKind,
        error: BGPUpdateMessageParsingError,
    ) -> Self {
        LocatedBGPUpdateMessageParsingError::new(input, error)
    }
}

/// Helper function to parse the withdraw routes buffer in an update message
#[inline]
fn parse_withdraw_routes(
    buf: Span<'_>,
) -> IResult<Span<'_>, Vec<WithdrawRoute>, LocatedBGPUpdateMessageParsingError<'_>> {
    let (buf, routes) = parse_till_empty_into_located(buf)?;
    Ok((buf, routes))
}

impl<'a> ReadablePDUWithOneInput<'a, bool, LocatedBGPUpdateMessageParsingError<'a>>
    for BGPUpdateMessage
{
    fn from_wire(
        buf: Span<'a>,
        asn4: bool,
    ) -> IResult<Span<'a>, Self, LocatedBGPUpdateMessageParsingError<'a>> {
        let (buf, withdrawn_buf) = nom::multi::length_data(be_u16)(buf)?;
        let (_, withdrawn_routes) = parse_withdraw_routes(withdrawn_buf)?;
        let (buf, path_attributes_buf) = nom::multi::length_data(be_u16)(buf)?;
        let (_, path_attributes) =
            parse_till_empty_into_with_one_input_located(path_attributes_buf, asn4)?;
        let (buf, nlri_vec) = parse_till_empty_into_located(buf)?;
        Ok((
            buf,
            BGPUpdateMessage::new(withdrawn_routes, path_attributes, nlri_vec),
        ))
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum WithdrawRouteParsingError {
    /// Errors triggered by the nom parser, see [nom::error::ErrorKind] for
    /// additional information.
    NomError(ErrorKind),
    Ipv4PrefixParsingError(Ipv4PrefixParsingError),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LocatedWithdrawRouteParsingError<'a> {
    span: Span<'a>,
    error: WithdrawRouteParsingError,
}

impl<'a> LocatedWithdrawRouteParsingError<'a> {
    pub const fn new(span: Span<'a>, error: WithdrawRouteParsingError) -> Self {
        Self { span, error }
    }
}

impl<'a> LocatedParsingError<'a, WithdrawRouteParsingError>
    for LocatedWithdrawRouteParsingError<'a>
{
    fn span(&self) -> &Span<'a> {
        &self.span
    }

    fn error(&self) -> &WithdrawRouteParsingError {
        &self.error
    }
}

impl<'a> IntoLocatedError<'a, BGPUpdateMessageParsingError, LocatedBGPUpdateMessageParsingError<'a>>
    for LocatedWithdrawRouteParsingError<'a>
{
    fn into_located(self) -> LocatedBGPUpdateMessageParsingError<'a> {
        LocatedBGPUpdateMessageParsingError::new(
            self.span,
            BGPUpdateMessageParsingError::WithdrawRouteError(self.error),
        )
    }
}

impl<'a> FromExternalError<Span<'a>, WithdrawRouteParsingError>
    for LocatedWithdrawRouteParsingError<'a>
{
    fn from_external_error(
        input: Span<'a>,
        _kind: ErrorKind,
        error: WithdrawRouteParsingError,
    ) -> Self {
        LocatedWithdrawRouteParsingError::new(input, error)
    }
}

impl<'a> ReadablePDU<'a, LocatedWithdrawRouteParsingError<'a>> for WithdrawRoute {
    fn from_wire(buf: Span<'a>) -> IResult<Span<'a>, Self, LocatedWithdrawRouteParsingError<'a>> {
        let (buf, net) = match ipv4_network_from_wire(buf) {
            Ok((buf, net)) => (buf, net),
            Err(err) => {
                return match err {
                    nom::Err::Incomplete(needed) => Err(nom::Err::Incomplete(needed)),
                    nom::Err::Error(error) => Err(nom::Err::Error(
                        error.into_located_bgp_withdraw_route_parsing_error(),
                    )),
                    nom::Err::Failure(failure) => Err(nom::Err::Failure(
                        failure.into_located_bgp_withdraw_route_parsing_error(),
                    )),
                }
            }
        };
        Ok((buf, WithdrawRoute::new(net)))
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NetworkLayerReachabilityInformationParsingError {
    /// Errors triggered by the nom parser, see [nom::error::ErrorKind] for
    /// additional information.
    NomError(ErrorKind),
    Ipv4PrefixParsingError(Ipv4PrefixParsingError),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct LocatedNetworkLayerReachabilityInformationParsingError<'a> {
    span: Span<'a>,
    error: NetworkLayerReachabilityInformationParsingError,
}

impl<'a> LocatedNetworkLayerReachabilityInformationParsingError<'a> {
    pub const fn new(
        span: Span<'a>,
        error: NetworkLayerReachabilityInformationParsingError,
    ) -> Self {
        Self { span, error }
    }
}

impl<'a> LocatedParsingError<'a, NetworkLayerReachabilityInformationParsingError>
    for LocatedNetworkLayerReachabilityInformationParsingError<'a>
{
    fn span(&self) -> &Span<'a> {
        &self.span
    }

    fn error(&self) -> &NetworkLayerReachabilityInformationParsingError {
        &self.error
    }
}

impl<'a> IntoLocatedError<'a, BGPUpdateMessageParsingError, LocatedBGPUpdateMessageParsingError<'a>>
    for LocatedNetworkLayerReachabilityInformationParsingError<'a>
{
    fn into_located(self) -> LocatedBGPUpdateMessageParsingError<'a> {
        LocatedBGPUpdateMessageParsingError::new(
            self.span,
            BGPUpdateMessageParsingError::NetworkLayerReachabilityInformationError(self.error),
        )
    }
}

impl<'a> FromExternalError<Span<'a>, NetworkLayerReachabilityInformationParsingError>
    for LocatedNetworkLayerReachabilityInformationParsingError<'a>
{
    fn from_external_error(
        input: Span<'a>,
        _kind: ErrorKind,
        error: NetworkLayerReachabilityInformationParsingError,
    ) -> Self {
        LocatedNetworkLayerReachabilityInformationParsingError::new(input, error)
    }
}

fn parse_nlri_ipv4(
    buf: Span<'_>,
) -> IResult<Span<'_>, Ipv4Net, LocatedNetworkLayerReachabilityInformationParsingError<'_>> {
    let (buf, net) = match ipv4_network_from_wire(buf) {
        Ok((buf, net)) => (buf, net),
        Err(err) => {
            return match err {
                nom::Err::Incomplete(needed) => Err(nom::Err::Incomplete(needed)),
                nom::Err::Error(error) => {
                    Err(nom::Err::Error(error.into_located_nlri_parsing_error()))
                }
                nom::Err::Failure(failure) => {
                    Err(nom::Err::Failure(failure.into_located_nlri_parsing_error()))
                }
            }
        }
    };
    Ok((buf, net))
}
impl<'a> ReadablePDU<'a, LocatedNetworkLayerReachabilityInformationParsingError<'a>>
    for NetworkLayerReachabilityInformation
{
    fn from_wire(
        buf: Span<'a>,
    ) -> IResult<Span<'a>, Self, LocatedNetworkLayerReachabilityInformationParsingError<'a>> {
        let mut buf = buf;
        let mut nets = vec![];
        while !buf.is_empty() {
            let (t, net) = parse_nlri_ipv4(buf)?;
            nets.push(net);
            buf = t;
        }
        Ok((buf, NetworkLayerReachabilityInformation::new(nets)))
    }
}