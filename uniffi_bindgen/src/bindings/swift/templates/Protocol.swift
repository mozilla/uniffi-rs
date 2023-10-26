public protocol {{ protocol_name }}: AnyObject {
    { % for meth in methods.iter() -% }
    { %- include "FunctionDocsTemplate.swift" % }
    func {{ meth.name() | fn_name }}({ % call swift:: arg_list_protocol(meth) % }) { % call swift:: async(meth) -% } { % call swift:: throws (meth) -% }
    { %- match meth.return_type() -% }
    { %- when Some with(return_type) % } -> {{ return_type | type_name - }}
    { %- else -% }
    { %- endmatch % }
    { % endfor % }
}
