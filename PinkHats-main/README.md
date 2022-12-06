# MediVault

A secure storage for keeping medical records.
The purpose of this project is to make the sensitive medical data accessible from
anywhere and give the right to the verified actors to read and write blockchain data.

It would be used as a "secrets vault" which would allow anybody to
store/share secrets. Furthermore, a group of the users (in our case e.g. doctors)
have the privilege to verify the content of secrets.

## Setup

The project is forked from the [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template).
For a detailed setup, refer to the [original documentation](https://docs.substrate.io/quick-start/).

## Project Overview

There are two pallets involved in the project. Pallet 1 is responsible for the user management
and storing medical records with the following main functions:
* `create_account`
* `patient_adds_record`
* `doctor_adds_record`
* `doctor_verifies_record`
* `share_record_with`

The seconds pallet is implementing the functionality of sharing records with other users, which is being called
from the first pallet via the `share_record_with` function.

The two pallets are tightly coupled.

### Sequence Diagram

A detailed sequence diagram to showcase the workflow of the event processing in the system.

![sequence-diagramm](https://user-images.githubusercontent.com/61457816/204662265-8133ad23-5db1-45f7-8ae6-6d6ba99f9f5f.png)

Walk-through of the workflow:

* User generates a public/private keypair off-chain. The public key of the users
is shared amongst each other, which is necessary if they want to share records.
* User can create an account, where the user type is specified. As for now, the types are defined
as `Pateint` and `Doctor`.
* Both types of users can add a record.
  * If a `Patient` adds the record, it is going to be unverified.
  * If a `Doctor` adds the record, it is going to be stored as a verified record, where the doctor's account ID acts as a signature.
* A `Doctor` can as well verify an unverified record.
* Users can share records with other users. The parameters of the function must
include:
  * the recipient's account ID, which is the recipient's public key (which has previously been generated off-chain, and been shared with the other user),
  * encrypted key, which is needed to decrypt the record (as a proof of work, sub-key was used to generate a clear text keypair to make decryption work)
  * record ID
* Finally, the recipient can query the record and decrypt it locally.
