# OpenSUSE Auth Moderator

This project has to solve an important problem - SUSE and OpenSUSE both share
resources, but they are also separate organisations with different needs. To
achieve a level of trust, we need a way to ensure that authentication can be
securely shared between both, meeting the strict security security requirements
that both organisations have.

Another objective of this project is that it should be invisible - it is a transparent
moderator that routes authentications to the relevant backend for SUSE and OpenSUSE
projects. Community users should never feel the impact of this project, and corporate
users should never have the ability to step outside corporate boundaries.

## Requirements

### No Credential Handling

The auth moderator *MUST* never see a users credential.

### Prevent Corporate Leaks

The auth moderator *MUST* prefer user-lookups from corporate over community.

The auth moderator should always ensure users of the corporate side always are forced
to corp auth and never community.

### Stateless Design

To prevent auth-tokens potentially being leaked, the auth moderator should be as stateless
as possible, never persisting a users token in a way that it can be recoverd or used by the
moderator. This also will allow for HA and failover.

## Design

```
                ┌─────────────────────────┐                 
                │                         │                 
                │         Service         │                 
                │                         │                 
                └─────────────────────────┘                 
                             │                              
                             │                              
                             ▼                              
                ┌─────────────────────────┐                 
                │                         │                 
                │     Auth Moderator      │                 
                │                         │                 
                └─────────────────────────┘                 
                             │                              
             ┌───────────────┴────────────────┐             
             │                                │             
             ▼                                ▼             
┌─────────────────────────┐      ┌─────────────────────────┐
│                         │      │                         │
│        SUSE Auth        │      │      OpenSUSE Auth      │
│                         │      │                         │
└─────────────────────────┘      └─────────────────────────┘
```

Services that need to consume auth from both SUSE and OpenSUSE communicate with
this auth moderator instead. The auth moderator then will route the authentication
request as required. Additionally, the auth moderator can also route information
requests to the required services as well. For example a service may need to
request SSH keys for a user, and it can request these from the moderator.

### OIDC

The primary method of authentication will be OIDC as this allows each authentication
provider to correctly and securely perform MFA.

To achieve this the moderator will present itself as an OIDC Server, while acting as
an OIDC client to the AUTH backends.

```
┌───────────────────┐      1. "I want       ┌─────────────┐                              
│                   ├───────to login"──────▶│             │                              
│                   │                       │     OBS     │                              
│            ┌──────┼──────────────────────▶│             │                              
│            │    ┌─┼──────Redirect─────────│             │                              
│            │    │ │                       └─────────────┘                              
│            │    │ │                          9. Query                                  
│        8. Token │ │                              ▼                                     
│            │    │ │                       ┌───────────────────┐                        
│            │    │ │                       │                   │──────────┐             
│            │    └─┼──────────────────────▶│                   │          │             
│       User └──────┼───────────────────────│ OIDC/Auth Router  │  3. Decide if user is  
│            ┌──────┼──────────────────────▶│                   │    Corp OR Community   
│            │    ┌─┼─────4. Redirect───────│                   │          │             
│            │    │ │                       │                   │◀─────────┘             
│            │    │ │                       └───────────────────┘                        
│            │    │ │                          10 Query                                  
│       7. Return │ │                              ▼                                     
│            │    │ │                       ┌───────────────┐                            
│            │    └─┼──────────────────────▶│               │                            
│            │      │◀────5. Do the auth────│    Corp OR    │                            
│            │      │──6. Here is my auth──▶│   community   │                            
│            └──────┼───────────────────────│               │                            
└───────────────────┘                       └───────────────┘                            
```

### Routing

Routing should be performed in a way that is transparent, such that the user always is routed
to the correct auth provider.

This can be easily achieved by examining the source IP of the client - users of the SUSE corporate
VPN and networks will be routed to corporate auth, and users outside of these ranges will be
routed to community auth.

This is an effective method of routing, since corporate SUSE users can only access SUSE auth when
they are within the corporate VPN. If they are within the VPN, they should be using a corporate
account. If they are not within the corporate VPN, there is no way to use corporate auth, only
community, so they should only see community auth.

To prevent a corporate user going to the community auth when the VPN is disabled, we will set
a cookie on the users browser. If the user attempts to go through the moderator with the taint
cookie enabled, and they are *not* on a coporate IP range they will either:

* Be given an error (To hinder corporate users from having a community account)
* Be given a warning that they are not on the VPN, and that if they wish to proceed they can use community auth instead.

