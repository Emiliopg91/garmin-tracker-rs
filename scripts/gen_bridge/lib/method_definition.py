from .commons import STANDARD_TYPE_ASSOC, GenericType
from dataclasses import dataclass
from pathlib import Path

@dataclass
class MethodDefinition:
    name: str
    params: list[()]
    ret_type: GenericType
    defined_at: str

    @staticmethod
    def get_definitions(base_path: Path, path: Path) -> list[MethodDefinition]:
        with open(path, "r",encoding="utf-8") as librs:
            orig_content = content = librs.read()


        methods = []
        while True:
            pos = content.find("#[tauri::command]")
            if pos < 0:
                break
            content = content[pos+18:]
            while not content.startswith("fn "):
                if content.startswith("async"):
                    content = content[6:]
            pos = orig_content[0:orig_content.find(content)].count("\n")+1
            content = content[3:]
            method=content.split("(")[0].strip()
            content = content[len(method):]
            close_idx=content.find("{")
            signature = content[:close_idx]

            ret = "None"
            if "->" in signature:
                ret = signature.split(" -> ")[1].strip()
            params_str = signature.split(" -> ")[0].strip().replace("(","").replace(")","")
            params = []
            for param in params_str.split(","):
                param=param.strip()
                if param:
                    parts= param.split(":")
                    parts[0]=parts[0].strip()
                    parts[1]=parts[1].strip()
                    if parts[1].startswith("&"):
                        parts[1] = parts[1][1:]
                    if "AppHandle" not in parts[1]:
                        params.append((parts[0].strip(),GenericType.from_str(parts[1].strip())))

            methods.append(MethodDefinition(method,params,GenericType.from_str(ret),f"{path.relative_to(base_path)}:{pos}"))

            content = content[close_idx+1:]

        return methods

    def get_custom_types(self):
        types = []
        for p in self.params:
            types.append(p[1])
        if self.ret_type:
            types.append(self.ret_type)

        res = []
        for idx,t in enumerate(types):
            if isinstance(t,str):
                res.append(t)
            else:
                for tt in t.get_inner_types():
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

        if isinstance(self.ret_type, str):
            ret = self.ret_type
        else:
            ret = self.ret_type.to_typescript()

        params = []
        for p in self.params:
            typ = p[1]
            if isinstance(typ,GenericType):
                typ=typ.to_typescript()
            params.append(p[0]+": "+typ)

        payload = ""
        if len(self.params)>0:
            names = []
            for p in self.params:
                names.append(p[0])
            payload=f", {{ {", ".join(names)} }}"

        result_lines = []
        result_lines.append(f"\t// Declaration: {self.defined_at}")
        result_lines.append(f"\tpublic static {ts_name}({", ".join(params)}): Promise<{ret}> {{")
        result_lines.append(f'\t\treturn RustBridge.inner_invoke("{self.name}"{payload});')
        result_lines.append("\t}")
        return "\n".join(result_lines)
