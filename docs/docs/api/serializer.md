# Serializer

The `serializer` submodule validates input and converts between JSON data and Python objects — typically SQLAlchemy models.

## Serializer

### Constructor

```python
Serializer(
    data: str | None = None,
    instance: Any | None = None,
    required: bool = True,
    nullable: bool = False,
    many: bool = False,
    context: dict | None = None,
    read_only: bool = False,
    write_only: bool = False,
)
```

- `data` — raw JSON string to validate
- `instance` — object (or list of objects when `many=True`) to serialize
- `context` — arbitrary dict available as `self.context` in overridden methods
- `many` — handle a list of objects

### Properties

| Property | Description |
| --- | --- |
| `instance` | The instance being serialized (settable) |
| `validated_data` | Validated fields after `is_valid()` |
| `raw_data` | The raw JSON string input (settable) |
| `context` | Arbitrary context passed to validation |
| `data` | Serialized representation of the instance(s); excludes `write_only` fields. `None` when no instance is set |

### Methods

| Method | Description |
| --- | --- |
| `is_valid()` | Parse `raw_data` and validate it; raises `ValidationException` on failure |
| `validate(attr: dict) -> dict` | Validate a Python dict; base implementation strips `read_only` fields |
| `schema() -> dict` | Generate the JSON Schema for the serializer |
| `create(session, validated_data)` | Build `Meta.model(**validated_data)`, `session.add`, `commit`, `refresh`; returns the instance |
| `save(session)` | Requires `validated_data` (call `is_valid()` first); calls `create(session, validated_data)` |
| `update(session, instance, validated_data)` | Set attributes on `instance`, commit, refresh |
| `to_representation(instance) -> dict` | Convert an instance to a dict using SQLAlchemy inspection |

## Validation flow

`is_valid()` reads `raw_data` (raising `ValidationException` when it is empty), parses the JSON string, then calls `validate(...)` — so a Python override of `validate` is honored. The base `validate()` checks the data against the JSON schema generated from the declared fields (including format validation) and removes `read_only` fields. The result is stored in `validated_data`.

```python
from oxapy import serializer


class Cred(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)


cred = Cred('{"email": "test@gmail.com", "password": "password"}')
cred.is_valid()
print(cred.validated_data)
```

When overriding `validate`, call `super().validate(attr)` to keep schema validation and `read_only` stripping:

```python
def validate(self, attr: dict) -> dict:
    attr["principal_amount"] = Decimal(attr["principal_amount"])
    return super().validate(attr)
```

## Model binding

For `create()` and `save()` to work, declare the model in a `Meta` class:

```python
class UserSerializer(serializer.Serializer):
    email = serializer.EmailField()
    password = serializer.CharField(min_length=8)

    class Meta:
        model = User
```

`create()` then:

1. Builds the instance: `Meta.model(**validated_data)`
2. `session.add(instance)`
3. `session.commit()`
4. `session.refresh(instance)`

and returns the instance. `save(session)` reads `validated_data` (raising `Exception` when `is_valid()` was not called first) and delegates to `create(session, validated_data)`, so overridden `create` methods are honored.

## Serialization with SQLAlchemy

`to_representation(instance)` uses SQLAlchemy's inspection API:

- It iterates the model's mapped **columns** and includes each value whose name matches a declared field, skipping `write_only` fields.
- It iterates the model's **relationships**; for each one that matches a declared nested serializer field, it sets the field's `context` and `instance` and stores the nested serializer's `data` (a list when `many=True`).

Override it to add computed values:

```python
def to_representation(self, instance: Loan):
    data = super().to_representation(instance)
    data.update({"principal_amount": float(data["principal_amount"])})
    return data
```

## Fields

### Field types

| Field | Purpose |
| --- | --- |
| `CharField` | Strings |
| `IntegerField` | Integers |
| `NumberField` | Floats / numbers |
| `EmailField` | Email addresses |
| `BooleanField` | Booleans |
| `DateField` | Dates |
| `DateTimeField` | Datetimes |
| `EnumField` | Restricted value sets |
| `UUIDField` | UUIDs |

### Field options

| Option | Description |
| --- | --- |
| `required` | Field must be present (default `True`) |
| `nullable` | `None` is allowed (default `False`) |
| `many` | Field holds a list |
| `length` | Exact length |
| `min_length` / `max_length` | Length bounds |
| `pattern` | Regex pattern |
| `enum_values` | Allowed values for `EnumField` |
| `format` | Value format |
| `read_only` | Excluded from validation/deserialization |
| `write_only` | Excluded from serialization |

Fields are regular Python classes, so they can be subclassed for domain-specific validation:

```python
class PhoneNumberSerializer(serializer.CharField):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.pattern = r"^(?:\+261|0)(32|33|34|37|38)\d{7}$"
```

## ValidationException

Raised by `is_valid()` when the input is missing or invalid.

## Related

- [Serializers guide](../guides/serializers) — full walkthrough with production patterns
- [Requests](./request) — reading JSON bodies
