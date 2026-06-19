from pathlib import Path
from dataclasses import dataclass
from .commons import STANDARD_TYPE_ASSOC,GenericType,GenericWrapper

@dataclass
class StructDefinition:
    name: str
    fields: dict[str,str|GenericType]
    defined_at: str

    def __eq__(self, other):
        return isinstance(other, StructDefinition) and self.name == other.name

    def __hash__(self):
        return hash(self.name)
    
    @staticmethod
    def get_definitions(base_dir:  Path,file: Path):
        with open(file, "r",encoding="utf-8") as librs:
            orig_content = content = librs.read()

        definitions: list[StructDefinition] = []
        while True:
            pos = content.find("pub struct ")
            if pos < 0:
                break

            defined_at = orig_content[0:orig_content.find(content[pos:])].count("\n")+1
            content = content[pos+11:]
            pos = content.find("{")
            name = content[:pos].strip()
            content = content[pos+1:]
            pos = content.find("}")
            field_dec = content[:pos].split("\n")
            fields={}
            for f in field_dec:
                f = f.strip()
                if f:
                    if f.startswith("pub "):
                        f = f[4:]
                    if f.endswith(","):
                        f=f[:-1]
                    parts = f.split(":")
                    param = parts[0]
                    typ = GenericType.from_str(parts[1].strip())
                    fields[param]=typ
            
            definition = StructDefinition(name,fields,f"{file.relative_to(base_dir)}:{defined_at}")
            definitions.append(definition)
            content = content[pos+1:]

        return definitions

    def get_custom_types(self):
        res = []
        for (_,typ) in self.fields.items():
            if isinstance(typ,str):
                if typ not in STANDARD_TYPE_ASSOC.keys():
                    res.append(typ)
            else:
                if typ:
                    for tt in typ.get_inner_types():
                        if tt not in STANDARD_TYPE_ASSOC.keys():
                            res.append(tt)

        return res
    


    def to_typescript(self):
        result_lines = []
        result_lines.append("// Definition: "+self.defined_at)
        result_lines.append(f"export interface {self.name} {{")
        for (name,typ) in self.fields.items():
            if isinstance(typ,GenericType):
                typ=typ.to_typescript()
            else:
                if typ in STANDARD_TYPE_ASSOC.keys():
                    typ = STANDARD_TYPE_ASSOC[typ]
            result_lines.append(f"\t{name}: {typ};")
        result_lines.append("}")
        return "\n".join(result_lines)

    @staticmethod
    def generate_file(file, structs):
        result_lines = []
        result_lines.append('/* eslint-disable */')
        result_lines.append('')
        result_lines.append('//Auto generated file, do not edit manually')
        result_lines.append("")

        for struct in sorted(structs, key=lambda x: x.name):
            result_lines.append(struct.to_typescript())
            result_lines.append("")

        with open(file, "w", encoding="utf-8") as f:
            f.write("\n".join(result_lines))