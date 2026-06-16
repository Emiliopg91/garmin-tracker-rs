from .commons import STANDARD_TYPE_ASSOC, GenericType
from dataclasses import dataclass
from pathlib import Path

@dataclass
class MethodDefinition:
    name: str
    params: list[()]
    ret_type: str

    @staticmethod
    def get_definitions(path: Path) -> list[MethodDefinition]:
        with open(path, "r",encoding="utf-8") as librs:
            lines = librs.read().splitlines()
            
        methods = []
        for idx in range(0,len(lines)): #pylint: disable=consider-using-enumerate
            line = lines[idx]
            if line.startswith("#[tauri::command]"):
                idx+=1
                line = lines[idx]
                if line.startswith("async "):
                    line = line[6:]
                method=line[3:].split("(")[0].strip()
                signature = ("("+line[3:].split("(")[1]+(" ".join(lines[idx+1:]))).split("{")[0]
                sig_parts = signature.split("->")
                params_str = sig_parts[0].strip()[1:-1]
                params = []
                if len(params_str)>0:
                    for p in params_str.split(","):
                        p = p.strip()
                        parts = p.split(":")
                        name =parts[0].strip()
                        typ =parts[1].strip()
                        if not typ.startswith("AppHandle"):
                            params.append((name,typ))
                ret = None
                if len(sig_parts)>1:
                    ret = sig_parts[1].strip()
                    if ret.startswith("Result<"):
                        ret=None
                methods.append(MethodDefinition(method,params,ret))
        return methods

    def get_custom_types(self):
        types = []
        for p in self.params:
            types.append(p[1])
        if self.ret_type:
            types.append(self.ret_type)

        res = []
        for idx,t in enumerate(types):
            if t.startswith("&"):
                t = t[1:]
                types[idx] = t
            t = GenericType.from_str(t)
            if isinstance(t,str):
                if t not in STANDARD_TYPE_ASSOC.keys():
                    res.append(t)
            else:
                for tt in t.get_inner_types():
                    if tt not in STANDARD_TYPE_ASSOC.keys():
                        res.append(tt)

        return res

    def to_typescript(self):
        ts_name = ""
        next_upper = False
        for c in self.name:
            if c!="_":
                if not next_upper:
                    ts_name+=c
                else:
                    ts_name+=c.upper()
                    next_upper=False
            else:
                next_upper=True

        ret = "void"
        if self.ret_type:
            ret = self.ret_type
            if ret.startswith("Vec<"):
                ret = ret[4:-1]+"[]"

        params = []
        for p in self.params:
            typ = p[1]
            if typ.startswith("&"):
                typ = typ[1:]
            if typ in STANDARD_TYPE_ASSOC.keys():
                typ = STANDARD_TYPE_ASSOC[typ]
            params.append(p[0]+": "+typ)

        payload = ""
        if len(self.params)>0:
            names = []
            for p in self.params:
                names.append(p[0])
            payload=f", {{ {", ".join(names)} }}"

        result_lines = []
        result_lines.append(f"\tpublic static {ts_name}({", ".join(params)}): Promise<{ret}> {{")
        result_lines.append(f'\t\treturn invoke("{self.name}"{payload});')
        result_lines.append("\t}")
        return "\n".join(result_lines)
