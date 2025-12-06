class Multipart:
    boundary = "--------------------------735323031399963166993862"

    def __init__(self, form_data: dict):
        self.__form_data = form_data

    @property
    def data(self) -> str:
        body_parts = []

        for k, v in self.__form_data.items():
            body_parts.append(f"--{self.boundary}")
            body_parts.append(f'Content-Disposition: form-data; name="{k}"')
            body_parts.append("")
            body_parts.append(v)

        body_parts.append(f"--{self.boundary}--")
        return "\r\n".join(body_parts)

    @property
    def headers(self) -> dict:
        return {"Content-type": f"multipart/form-data; boundary={self.boundary}"}
