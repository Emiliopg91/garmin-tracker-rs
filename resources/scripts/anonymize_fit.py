#!/usr/bin/env python3
"""Anonymizes FIT files in place so they can be committed as test fixtures.

- GPS track (record, session and lap positions) is translated to a random city, preserving its shape.
- Garbage after the NUL terminator of string fields is wiped (Garmin leaks memory there, coordinates included).
- Paired sensor names are blanked.
- Device serial numbers are replaced by a fixed fake one.
- User profile height, weight and resting HR are replaced by fixed values.

Everything else is left byte-for-byte untouched and the CRCs are recomputed.

Usage: anonymize_fit.py [--city NAME] FILE.fit [FILE.fit ...]
"""
import argparse
import random
import struct
import sys

CITIES = {
    'oslo': (59.9139, 10.7522),
    'buenos_aires': (-34.6037, -58.3816),
    'kyoto': (35.0116, 135.7681),
    'reykjavik': (64.1466, -21.9426),
    'cape_town': (-33.9249, 18.4241),
    'vancouver': (49.2827, -123.1207),
    'melbourne': (-37.8136, 144.9631),
    'lisbon': (38.7223, -9.1393),
    'montreal': (45.5019, -73.5674),
    'krakow': (50.0647, 19.9450),
    'valparaiso': (-33.0472, -71.6127),
    'seoul': (37.5665, 126.9780),
}

SEMICIRCLES = 2**31 / 180
INVALID_SINT32 = 0x7FFFFFFF
STRING = 0x07

# (mesg, field) -> axis, for every field holding a semicircle position
POSITIONS = {
    # record: position
    (20, 0): 'lat', (20, 1): 'lon',
    # session: start, bounding box, undocumented end position
    (18, 3): 'lat', (18, 4): 'lon', (18, 29): 'lat', (18, 30): 'lon',
    (18, 31): 'lat', (18, 32): 'lon', (18, 38): 'lat', (18, 39): 'lon',
    # lap: start, end, bounding box
    (19, 3): 'lat', (19, 4): 'lon', (19, 5): 'lat', (19, 6): 'lon',
    (19, 27): 'lat', (19, 28): 'lon', (19, 29): 'lat', (19, 30): 'lon',
}
SERIALS = {(0, 3), (23, 3)}  # file_id / device_info serial_number
SENSOR_NAME = (147, 2)
FAKE_SERIAL = 3_000_000_001
USER_PROFILE = {3: 175, 4: 750, 8: 60}  # height (cm), weight (0.1 kg), resting HR

CRC_TABLE = [0x0000, 0xCC01, 0xD801, 0x1400, 0xF001, 0x3C00, 0x2800, 0xE401,
             0xA001, 0x6C00, 0x7800, 0xB401, 0x5000, 0x9C01, 0x8801, 0x4400]


def crc16(data):
    crc = 0
    for b in data:
        for nibble in (b & 0xF, b >> 4):
            tmp = CRC_TABLE[crc & 0xF]
            crc = ((crc >> 4) & 0x0FFF) ^ tmp ^ CRC_TABLE[nibble]
    return crc


def walk(buf):
    """Yields (mesg_num, field_num, base_type, offset, size, endian, is_dev) for every data field."""
    header_size = buf[0]
    end = header_size + struct.unpack_from('<I', buf, 4)[0]
    pos = header_size
    defs = {}
    while pos < end:
        rh = buf[pos]
        pos += 1
        if rh & 0x80:  # compressed timestamp header
            local, is_def = (rh >> 5) & 0x3, False
        else:
            local, is_def = rh & 0x0F, bool(rh & 0x40)

        if is_def:
            endian = '<' if buf[pos + 1] == 0 else '>'
            mesg = struct.unpack_from(endian + 'H', buf, pos + 2)[0]
            count = buf[pos + 4]
            pos += 5
            fields = []
            for _ in range(count):
                num, size, base = buf[pos:pos + 3]
                fields.append((num, base, size, False))
                pos += 3
            if rh & 0x20:  # developer fields
                count = buf[pos]
                pos += 1
                for _ in range(count):
                    num, size, dev_idx = buf[pos:pos + 3]
                    fields.append((num, dev_idx, size, True))
                    pos += 3
            defs[local] = (mesg, endian, fields)
        else:
            mesg, endian, fields = defs[local]
            for num, base, size, dev in fields:
                yield mesg, num, base, pos, size, endian, dev
                pos += size

    if pos != end:
        raise ValueError(f'malformed FIT file: data ends at {pos}, expected {end}')


def first_position(buf, fields):
    lat = None
    for mesg, num, _, off, _, endian, dev in fields:
        if mesg != 20 or dev or num not in (0, 1):
            continue
        value = struct.unpack_from(endian + 'i', buf, off)[0]
        if num == 0:
            lat = value
        elif lat is not None and INVALID_SINT32 not in (lat, value):
            return lat, value
    return None


def anonymize(path, city):
    buf = bytearray(open(path, 'rb').read())
    if len(buf) < 14 or buf[8:12] != b'.FIT':
        raise ValueError('not a FIT file')
    if crc16(buf) != 0:
        raise ValueError('bad CRC')

    fields = list(walk(buf))
    origin = first_position(buf, fields)
    delta = None
    if origin:
        delta = {
            'lat': round(city[0] * SEMICIRCLES) - origin[0],
            'lon': round(city[1] * SEMICIRCLES) - origin[1],
        }

    for mesg, num, base, off, size, endian, dev in fields:
        if dev:
            continue
        key = (mesg, num)
        if base == STRING:
            nul = 0 if key == SENSOR_NAME else buf.find(0, off, off + size) - off
            if nul >= 0:
                buf[off + nul:off + size] = bytes(size - nul)
        elif key in POSITIONS and size == 4:
            value = struct.unpack_from(endian + 'i', buf, off)[0]
            if value != INVALID_SINT32:
                if delta is None:
                    raise ValueError(f'position in mesg {mesg} but no record position to anchor it')
                struct.pack_into(endian + 'i', buf, off, value + delta[POSITIONS[key]])
        elif key in SERIALS and size == 4:
            if struct.unpack_from(endian + 'I', buf, off)[0] not in (0, 0xFFFFFFFF):
                struct.pack_into(endian + 'I', buf, off, FAKE_SERIAL)
        elif mesg == 3 and num in USER_PROFILE and size in (1, 2):
            struct.pack_into(endian + ('B' if size == 1 else 'H'), buf, off, USER_PROFILE[num])

    if buf[0] >= 14:
        struct.pack_into('<H', buf, 12, crc16(buf[:12]))
    struct.pack_into('<H', buf, len(buf) - 2, crc16(buf[:-2]))

    with open(path, 'wb') as f:
        f.write(buf)
    return origin is not None


def main():
    parser = argparse.ArgumentParser(description='Anonymize FIT files in place.')
    parser.add_argument('--city', choices=sorted(CITIES), help='target city (random per file by default)')
    parser.add_argument('files', nargs='+')
    args = parser.parse_args()

    failed = False
    for path in args.files:
        name = args.city or random.choice(sorted(CITIES))
        try:
            moved = anonymize(path, CITIES[name])
        except (OSError, ValueError, KeyError, struct.error) as e:
            print(f'{path}: {e}', file=sys.stderr)
            failed = True
            continue
        print(f'{path}: anonymized' + (f', track moved to {name}' if moved else ' (no GPS)'))
    sys.exit(1 if failed else 0)


if __name__ == '__main__':
    main()
