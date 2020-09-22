class {{ rec.name() }}(object):
    def __init__(self,{% for field in rec.fields() %}{{ field.name()|var_name_py }}{% if loop.last %}{% else %}, {% endif %}{% endfor %}):
        {%- for field in rec.fields() %}
        self.{{ field.name()|var_name_py }} = {{ field.name()|var_name_py }}
        {%- endfor %}

    def __str__(self):
        return "{{ rec.name() }}({% for field in rec.fields() %}{{ field.name() }}={}{% if loop.last %}{% else %}, {% endif %}{% endfor %})".format({% for field in rec.fields() %}self.{{ field.name() }}{% if loop.last %}{% else %}, {% endif %}{% endfor %})

    def __eq__(self, other):
        {%- for field in rec.fields() %}
        if self.{{ field.name()|var_name_py }} != other.{{ field.name()|var_name_py }}:
            return False
        return True
        {%- endfor %}

    @classmethod
    def _coerce(cls, v):
        # TODO: maybe we could do a bit of duck-typing here, details TBD
        assert isinstance(v, {{ rec.name() }})
        return v

    @classmethod
    def _lift(cls, rbuf):
        return cls._liftFrom(RustBufferStream(rbuf))

    @classmethod
    def _liftFrom(cls, buf):
        return cls(
            {%- for field in rec.fields() %}
            {{ "buf"|lift_from_py(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
            {%- endfor %}
        )

    @classmethod
    def _lower(cls, v):
        buf = RustBufferBuilder()
        try:
            cls._lowerInto(v, buf)
            return buf.finalize()
        except Exception:
            buf.discard()
            raise

    @classmethod
    def _lowerInto(cls, v, buf):
        {%- for field in rec.fields() %}
        {{ "(v.{})"|format(field.name())|lower_into_py("buf", field.type_()) }}
        {%- endfor %}
