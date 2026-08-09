# Serializers

The `serializer` submodule validates input and converts between JSON data and Python objects — typically SQLAlchemy models. It is modeled after Django REST Framework serializers.

## Defining a serializer

Subclass `Serializer` and declare fields as class attributes:

```python
from oxapy import serializer


class UserSerializer(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)
    age = serializer.IntegerField(nullable=True)
```

### Binding a model

For persistence (`create`/`save`) and for serialization of model instances, declare the model in an inner `Meta` class:

```python
class UserSerializer(serializer.Serializer):
    id = serializer.CharField(read_only=True, required=False, nullable=True)
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8, write_only=True)

    class Meta:
        model = User
```

`create(session, validated_data)` uses `Meta.model` to build the instance: it calls `Model(**validated_data)`, then `session.add(...)`, `session.commit()`, and `session.refresh(...)`.

### Available fields

| Field | Validates |
| --- | --- |
| `CharField` | strings, with `length` / `min_length` / `max_length` / `pattern` |
| `IntegerField` | integers |
| `NumberField` | floats / numbers |
| `EmailField` | email addresses |
| `BooleanField` | booleans |
| `DateField` | dates |
| `DateTimeField` | datetimes |
| `EnumField` | values from `enum_values` |
| `UUIDField` | UUIDs |

### Field options

All fields accept: `required`, `nullable`, `many`, `format`, `length`, `min_length`, `max_length`, `pattern`, `enum_values`, `read_only`, `write_only`.

### Custom field subclasses

Create domain-specific fields by subclassing a built-in field and setting options in `__init__`. For example, a phone number field with a fixed pattern:

```python
class PhoneNumberSerializer(serializer.CharField):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.pattern = r"^(?:\+261|0)(32|33|34|37|38)\d{7}$"


class CredentialSerializer(serializer.Serializer):
    phone_number = PhoneNumberSerializer()
    password = serializer.CharField(min_length=8)
```

## Validating input

Pass a JSON string as `data`, then call `is_valid()`:

```python
from oxapy import serializer


class Cred(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)


cred = Cred('{"email": "test@gmail.com", "password": "password"}')
cred.is_valid()
print(cred.validated_data)
# {'email': 'test@gmail.com', 'password': 'password'}
```

The flow is:

1. `is_valid()` parses the `raw_data` JSON string.
2. It calls `validate(...)` — the base implementation checks the data against the JSON schema generated from your fields (formats included) and removes any `read_only` fields.
3. The result is stored in `validated_data`.

Invalid input raises `serializer.ValidationException`:

```python
cred.raw_data = '{"email": "invalid", "password": "password"}'
cred.is_valid()  # raises serializer.ValidationException
```

### Overriding validate

Override `validate(attr)` to transform values before they are stored in `validated_data`. Call `super().validate(attr)` to keep schema validation and `read_only` stripping:

```python
from decimal import Decimal


class LoanSerializer(serializer.Serializer):
    principal_amount = serializer.CharField()
    interest_rate = serializer.CharField(required=False, read_only=True, nullable=True)
    state = serializer.EnumField(
        required=False,
        read_only=True,
        enum_values=["pending", "accepted", "refused", "repaid", "unpaid"],
    )

    class Meta:
        model = Loan

    def validate(self, attr: dict) -> dict:
        attr["principal_amount"] = Decimal(attr["principal_amount"])
        return attr
```

## Serializing model instances

Pass an instance and read `.data`. With a `Meta.model` set, the base `to_representation()` uses SQLAlchemy's inspection API: it walks the model's mapped columns (and relationships) and copies the values of every field you declared — skipping `write_only` fields.

```python
user = user_srvs.get_user_by_id(db, user_id)
serializer = UserSerializer(instance=user)
print(serializer.data)
```

Fields marked `write_only=True` are excluded from the output; `read_only=True` fields are excluded from validation:

```python
class AccountSerializer(serializer.Serializer):
    id = serializer.CharField(read_only=True, nullable=True, required=False)
    name = serializer.CharField()
    password = serializer.CharField(write_only=True)


acc = AccountSerializer('{"id": null, "name": "joe", "password": "password"}')
acc.is_valid()
print(acc.validated_data)  # {'name': 'joe', 'password': 'password'}
```

### Overriding to_representation

Decorate the serialized dict with computed values:

```python
def to_representation(self, instance: Loan):
    data = super().to_representation(instance)
    data.update(
        {
            "principal_amount": float(data["principal_amount"]),
            "interest_rate": float(data["interest_rate"]),
        }
    )
    return data
```

## Nested serializers and relationships

A serializer field can be another serializer. Combined with `many=True` it serializes to-one and to-many SQLAlchemy relationships; the base `to_representation()` picks up relationship values automatically and passes the parent `context` through to the nested serializer.

```python
class LoanSerializer(serializer.Serializer):
    id = serializer.CharField(required=False, read_only=True)
    principal_amount = serializer.CharField()
    state = serializer.EnumField(required=False, read_only=True, enum_values=["pending", "accepted"])

    class Meta:
        model = Loan


class MemberSerializer(serializer.Serializer):
    id = serializer.CharField(required=False, read_only=True)
    share_number = serializer.IntegerField()
    loans = LoanSerializer(required=False, read_only=True, many=True)

    class Meta:
        model = Member
```

```python
member = member_srvs.get_user_membership(db, user_id)
serializer = MemberSerializer(instance=member)
print(serializer.data)
# {'id': ..., 'share_number': 5, 'loans': [{...}, {...}]}
```

## Passing context

`context` is an arbitrary dict available as `self.context` inside overridden methods — useful for values that are not part of the request payload, like the acting user:

```python
class GroupSerializer(serializer.Serializer):
    name = serializer.CharField()
    creator_id = serializer.CharField(required=False, read_only=True)

    class Meta:
        model = Group

    def create(self, session, validated_data):
        user_id = self.context["user_id"]
        validated_data["id"] = new_id()
        validated_data["creator_id"] = user_id
        group = super().create(session, validated_data)
        session.commit()
        return group
```

```python
serializer = GroupSerializer(r.data, context={"user_id": r.user_id})
serializer.is_valid()
group = serializer.create(db, serializer.validated_data)
```

## Persisting with save

`save(session)` persists the validated data: it requires `validated_data` (call `is_valid()` first) and then calls `create(session, validated_data)` — so an overridden `create` is honored. The typical split is: validate in the handler, persist in the service layer.

```python
# handler
new_user = UserSerializer(r.data)
new_user.is_valid()
user_instance = user_srvs.register(r.db, new_user)

# service
def register(db, new_user: UserSerializer) -> User:
    return new_user.save(db)
```

:::note

`create()` calls `session.commit()` internally, and `update()` commits after setting the new attribute values on the instance. If you perform extra work in an overridden `create` (for example adding related rows), add those rows before a final `session.commit()`.

:::

## Updating instances

`update(session, instance, validated_data)` sets each validated field on the instance, commits, and refreshes:

```python
serializer = UserSerializer()
updated = serializer.update(db, user, {"email": "new@email.com"})
```

## Schema generation

`schema()` returns the JSON Schema derived from the serializer's fields, and `to_representation(instance)` is also available standalone. Both are handy for generating OpenAPI-style documentation.

## Using serializers in handlers

A complete round trip — validate a JSON body, create the row, and return it with a status code:

```python
from oxapy import Router, Status, post, serializer


class SignupSerializer(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)

    class Meta:
        model = User


@post("/signup")
def signup(request):
    new_user = SignupSerializer(request.data)
    new_user.is_valid()
    user_instance = new_user.save(request.db)          # request.db from a middleware
    return {"user": SignupSerializer(instance=user_instance).data}, Status.CREATED
```

## Next steps

- [Requests](./requests) — reading raw JSON bodies with `request.data`
- [Middleware](./middleware) — attaching a database session to `request.db`
- [API Reference: Serializer](../api/serializer) — full reference
