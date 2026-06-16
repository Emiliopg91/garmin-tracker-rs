from pathlib import Path
from dataclasses import dataclass
from .commons import STANDARD_TYPE_ASSOC

@dataclass
class StructDefinition:
    name: str
    fields: dict[str,str]

    def __eq__(self, other):
        return isinstance(other, StructDefinition) and self.name == other.name

    def __hash__(self):
        return hash(self.name)
    
    @staticmethod
    def get_definitions(file: Path):
        content = file.read_text(encoding="utf-8")
        lines = content.splitlines()

        definitions: list[StructDefinition] = []
        for idx in range(0,len(lines)): #pylint: disable=consider-using-enumerate
            line=lines[idx].strip()
            parts = line.split("pub struct ")
            if len(parts)>1:
                name = parts[1][:-2].strip()

                idx+=1
                fields = {}
                while not lines[idx].strip().startswith("}"):
                    line=lines[idx].strip()
                    if line.startswith("pub "):
                        line=line[4:]
                    if line.endswith(","):
                        line=line[:-1]
                    idx+=1
                    parts = line.split(":")
                    if len(parts)>1:
                        fields[parts[0]]=parts[1].strip()
                definition = StructDefinition(name,fields)
                definitions.append(definition)

        return definitions
    
    def get_custom_types(self):
        types = []
        for p in self.fields.keys():
            types.append(self.fields[p])

        res = []
        for idx,t in enumerate(types):
            if t.startswith("&"):
                t = t[1:]
                types[idx] = t
            if t.endswith(">"):
                t=t.split("<")[1].split(">")[0]
                if "," in t:
                    t=t.split(",")[1].strip()
            if t not in STANDARD_TYPE_ASSOC.keys():
                res.append(t)

        return res
    


    @staticmethod
    def __rust_to_ts(type_str: str) -> str:
        if "<" in type_str:
            template = type_str[0:type_str.find("<")]

            generics = type_str[type_str.find("<")+1:type_str.rfind(">")].split(", ")

            handled_generics = []
            for generic in generics:
                handled_generics.append(StructDefinition.__rust_to_ts(generic))

            if template=="Vec":
                return handled_generics[0]+"[]"

            if template.endswith("Map"):
                return "Record<"+handled_generics[0]+", "+handled_generics[1]+">"

            if template=="Option":
                return handled_generics[0]+" | null"

        if type_str in STANDARD_TYPE_ASSOC.keys():
            type_str = STANDARD_TYPE_ASSOC[type_str]

        return type_str

    def to_typescript(self):
        result_lines = []
        result_lines.append(f"export interface {self.name} {{")
        for (name,typ) in self.fields.items():
            typ = StructDefinition.__rust_to_ts(typ)
            result_lines.append(f"\t{name}: {typ}")
        result_lines.append("}")
        return "\n".join(result_lines)
