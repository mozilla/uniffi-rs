StringBuilder(){% for v in values %}.append({{ v }}){% endfor %}.toString()
