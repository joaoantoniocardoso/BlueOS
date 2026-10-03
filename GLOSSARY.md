# BlueOS

The operating system for marine robots, built as event-driven services that talk over one Zenoh backbone through a
versioned IDL API.

## Service structure

**Service**:
One process that runs one Domain inside a Kernel, and owns everything that lives as long as the process: the CLI
values, logging, the async runtime, the settings folder, the Zenoh Session, and the exit code.
_Avoid_: App, daemon, microservice (when meaning the Rust structure)

**Kernel**:
The part of a Service that runs the Domain: it owns everything that can be stopped (the Inbox, Tasks, timers, the
liveliness token) and is the only writer of the DomainState.
_Avoid_: runtime, framework, App

**Domain**:
The pure logic of a Service: it receives Commands, changes its Snapshot, and returns a Decision. It never does IO,
never waits, and never reads a clock.
_Avoid_: business logic, handler (for the whole), CQRS domain

**Block**:
A pure, reusable piece of logic that a Domain composes, with its own part of the Snapshot and its own Commands,
domain events, and IO requests. The Kernel never runs a Block directly.
_Avoid_: sub-domain, module, policy

**Sans-IO component**:
A pure piece of logic that is neither a Domain nor a Block: no Domain composes it, and Domains, Blocks, Tasks or
adapters call it, such as the gate that holds samples until their schema is known.
_Avoid_: library (collides with the recording library), helper, util

**DomainState**:
The Snapshot of a Service, including the Jobs it keeps: all the data the Domain owns, and everything that can be
copied.
_Avoid_: App, state (alone), store

**Snapshot**:
The data a Domain keeps between Commands.
_Avoid_: model, store, context

**Durable state**:
The part of a Snapshot, together with the Jobs, that a Service keeps across restarts. Unlike the Settings document,
it belongs to the Domain and is discarded rather than migrated when its version changes.
_Avoid_: checkpoint, saved state, cache

**Job**:
A unit of work a client submits to a Service and can watch, cancel, pause, resume and approve, modelled on a ROS 2
action. Every Request either submits a Job or controls one.
_Avoid_: task (for this meaning), workflow, job graph

**Job type**:
The kind of a Job, declared in code with its nature: the controls it allows, whether it needs permission, the
Resources it needs, how it reacts to a revoked Lease, its concurrency limit and its conflict key.
_Avoid_: job spec, job kind

**Goal**:
The parameters a client sends when it submits a Job, saying what the Job should achieve.
_Avoid_: request (for this meaning), arguments, payload

**Feedback**:
The progress a running Job reports while it executes.
_Avoid_: status (for this meaning), progress event

**Job result**:
What a Job reports once, when it ends.
_Avoid_: response, reply, outcome

**Permission request**:
A question a Job asks the user before it executes. Any client may answer it, and it expires at a deadline its Job
type sets.
_Avoid_: prompt, confirmation

**Task**:
A long-running piece of work that the Kernel starts, supervises, restarts, and stops, such as a data plane.
_Avoid_: job (for this meaning), background thread, spawn

**Inbox**:
The one ordered queue through which every Command reaches the Domain.
_Avoid_: mailbox, event loop, channel

**Context**:
The set of dependencies a Service hands to its IO code, handlers and Tasks: paths, devices, connections, the clock,
and its Ports. It is the only thing a test changes to vary what a Service runs with.
_Avoid_: container, IoContext, environment

**Port**:
A replaceable piece of a Context through which IO code reaches the outside world, filled with the real adapter in
production and replaceable by a test.
_Avoid_: hook, seam, injection point

## Domain vocabulary

**Command**:
An input to a Domain that may change the Snapshot: a client request, an IO result, a timer tick, or a fact observed
on the bus.
_Avoid_: action, message (for this meaning), request (for the whole)

**Request**:
A Command that a client sent through a Command endpoint, which submits a Job or controls one. It is the only kind of
Command that can come from outside the process.
_Avoid_: client command, external command

**IO result**:
A Command that reports how an Effect ended.
_Avoid_: callback, completion

**Tick**:
A Command that a scheduled Effect delivers when its time comes.
_Avoid_: timer event, timeout (as a noun for the Command)

**Observed fact**:
A Command that reports something a Task saw in the outside world, such as the vehicle arming.
_Avoid_: injected command, internal command

**Projection**:
A value derived purely from the Snapshot and kept current after every applied Command, so Tasks and States can follow
the Domain without asking it.
_Avoid_: selector, view (for this meaning), watch

**Outcome**:
What a Block returns for one Command: domain events, Effects, or a rejection.
_Avoid_: result, response

**Decision**:
The Outcome of the Domain itself, which the Kernel carries out.
_Avoid_: result, reply

**Effect**:
An order from the Domain to the Kernel to do something the Domain cannot: IO, or scheduling a later Command.
_Avoid_: side effect, action

**domain event**:
A fact a Domain reports about a change it made. It exists only inside the process until the Kernel publishes it as an
Event.
_Avoid_: notification, signal

**Control plane**:
The low-rate decisions about what a Service should do, owned by the Domain.

**Data plane**:
The high-rate path that moves payloads (recorded samples, video) according to the control plane, without passing
them through the Inbox.
_Avoid_: hot path (as a noun), pipeline

## Resources

**Resource**:
Anything a Job must hold while it executes: an amount it consumes (disk space, bandwidth), an exclusive device, or a
Condition.
_Avoid_: capability, lock

**Condition**:
A Resource that is a fact which must stay true while a Job executes, such as the vehicle being disarmed.
_Avoid_: precondition, guard

**Lease**:
The revocable right of one Job to hold one Resource.
_Avoid_: lock, reservation

**Arbiter**:
What grants and revokes Leases: each Service's own until the service manager takes the role.
_Avoid_: lock manager, scheduler

## API and wire

**Message**:
A type defined in the BlueOS IDL (a ROS 2 `.msg`) that travels on the backbone as CDR bytes.
_Avoid_: DTO, packet, payload (for the type)

**Session**:
A Service's connection to the Zenoh backbone.
_Avoid_: connection, client; never use it for a recording

**Command endpoint**:
A public entry point where a client submits a Job of one Job type, or controls an existing Job, and gets back
whether it was accepted.

**Query endpoint**:
A public entry point where a client asks a Service for information without changing anything.
_Avoid_: service (the ROS 2 name for it, which collides with Service)

**IO query endpoint**:
A Query endpoint answered by IO outside the Inbox, such as reading a device, instead of from the Snapshot. Clients
cannot tell it from any other Query endpoint.

**Endpoint manifest**:
The committed list of a Service's public endpoints, `app/endpoints.toml`: each one's kind, name, key and
interface type. The code that registers them is generated from it.
_Avoid_: routes, endpoint config

**Custom endpoint**:
The Command endpoint of a Job type whose Goal needs the Context to become a Request, so the Service maps it in
code it writes. Refusing an invalid Goal does not make an endpoint custom.

**State**:
A public value a Service keeps current: published on change and readable at any time by late clients.

**Event**:
A public notice that something happened, published once and not kept.
_Avoid_: domain event (for the published form)

**Settings document**:
The persisted, versioned configuration of one Service, in the same format the Python services use.
_Avoid_: config, policy, preferences
