from oxapy import serializer


class User(serializer.Serializer):
    name = serializer.Field("string")


class Product(serializer.Serializer):
    owner = User()


product = Product(request)

print(product.valide())
