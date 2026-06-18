from dataclasses import dataclass
from enum import Enum

STANDARD_TYPE_ASSOC : dict[str,str] ={
    "bool":"boolean",
    "i8":"number",
    "i16":"number",
    "i32":"number",
    "i64":"string",
    "isize":"number",
    "u8":"number",
    "u16":"number",
    "u32":"number",
    "u64":"number",
    "f32":"number",
    "f64":"number",
    "usize":"number",    
    "str": "string",
    "String": "string",
    "None": "void",
    "()": "void"
}

class GenericWrapper(Enum):
    VEC=0
    MAP=1
    OPTION=2
    RESULT =3

@dataclass
class GenericType:
    wrapper: GenericWrapper
    types: str|list[GenericType]
    strings: list[str]

    @staticmethod
    def from_str(type_str: str) -> str|GenericType:
        if "<" not in type_str:
            if type_str.startswith("&"):
                type_str = type_str[1:]
            return type_str

        template = type_str[0:type_str.find("<")]

        generics = type_str[type_str.find("<")+1:type_str.rfind(">")].split(",")

        handled_generics = []
        strings=[]
        for generic in generics:
            strings.append(generic.strip())
            handled_generics.append(GenericType.from_str(generic.strip()))

        if template=="Vec":
            return GenericType(GenericWrapper.VEC,[handled_generics[0]],strings)

        if template.endswith("Map"):
            return GenericType(GenericWrapper.MAP,[handled_generics[0],handled_generics[1]],strings)

        if template=="Option":
            return GenericType(GenericWrapper.OPTION,[handled_generics[0]],strings)

        if template=="Result":
            return GenericType(GenericWrapper.RESULT,[handled_generics[0], handled_generics[1]],strings)

    def to_typescript(self):
        if isinstance(self.types, str):
            if self.types in STANDARD_TYPE_ASSOC.keys():
                self.types = STANDARD_TYPE_ASSOC[self.types]
            return self.types

        if isinstance(self.types[0],str):
            handled_0 = self.types[0]
            if handled_0 in STANDARD_TYPE_ASSOC.keys():
                handled_0 = STANDARD_TYPE_ASSOC[handled_0]
        else:
            handled_0 = self.types[0].to_typescript()

        handled_1=None
        if len(self.types)>1:
            if isinstance(self.types[1],str):
                handled_1 = self.types[1]
            if handled_1 in STANDARD_TYPE_ASSOC.keys():
                handled_1 = STANDARD_TYPE_ASSOC[handled_1]
            else:
                handled_1 = self.types[1].to_typescript()

        if self.wrapper==GenericWrapper.VEC:
            return handled_0+"[]"
        if self.wrapper==GenericWrapper.MAP:
            return "Record<"+handled_0+", "+handled_1+">"
        if self.wrapper==GenericWrapper.OPTION:
            return handled_0+" | null"
        if self.wrapper==GenericWrapper.RESULT:
            return handled_0

    def get_inner_types(self):
        types=[]
        for t in self.types:
            if isinstance(t,str):
                types.append(t)
            else:
                for tt in t.get_inner_types():
                    types.append(tt)
        return types
