# Logic for potential values to generate NFT Json
#
# 1. Equipment
#     1. Level: 30 --> 1...30
#     2. Item Feature: 5 --> Power, Magic, Fire, Earth, Water,
#     3. Item Rarity: 4 --> Common, Rare, Epic, Legendary
#     4. Equipment type: 3 --> Head, Robe, Staff
#     5. Value
#         1. Percent (Power and Magic): 1..1400
#             1. min = 100
#             2. max = (item_level / 3 * 10) + (multiplier * 10);
#                3. multiplier --> common = 10, rare = 20, epic = 30, legendary = 40
#         2. Number (Fire, earth, water): 1..120
#             1. min = item_level * (multiplier - 10) + 1;
#             2. max = item_level * multiplier;
#                3. multiplier --> common = 10, rare = 20, epic = 30, legendary = 40
#
# 2. Chest
#     1. Level: 30 --> 1...30
#     2. Tier: 4 --> 1..4 (Based on level)
#
# 3. Spellbook
#     1. Level: 30 --> 1...30
#     2. Spell type: 6 -> Fire, Water, Earth, Experience, Craft, Item
#     3. Cost feature: 3 -> Fire, Earth, Water
#     4. Rarity: 4 -> Common, Rare, Epic, Legendary
#     5. Cost: 1..300
#         1. min = item level
#         2. max = item level * 10
#     6. Value: 1..3600
#         1. Experience
#             1. ((cost * (rarity_odds - 1)) + (SPELL_MULTIPLE * avg_resources)) * multiplier
#                 1. cost --> item cost
#                 2. rarity_odds --> common = 8, rare = 6, epic = 4, legendary = 2
#                 3. SPELL_MULTIPLE --> 4
#                 4. avg_resources = round (rarity min *  rarity max  / 2)
#                     1. rarity min --> common = 1, rare = 11 , epic = 21, legendary = 31
#                     2. rarity max --> common = 10, rare = 20, epic = 30, legendary = 40
#                 5. multiplier = 2
#         2. Earth, Fire, Water
#             1. ((cost * (rarity_odds - 1)) + (SPELL_MULTIPLE * avg_resources)) * multiplier
#                 1. cost --> item cost
#                 2. rarity_odds --> common = 8, rare = 6, epic = 4, legendary = 2
#                 3. SPELL_MULTIPLE --> 4
#                 4. avg_resources = round (rarity min *  rarity max  / 2)
#                     1. rarity min --> common = 1, rare = 11 , epic = 21, legendary = 31
#                     2. rarity max --> common = 10, rare = 20, epic = 30, legendary = 40
#                 5. multiplier = 1
#         3. Item, Craft
#             1. 0
import copy
import json
import os

CREATOR_PUBLIC_KEY = "LCz1zm4nzTPLpfkC7y4t7Vr2RTEDGow7t3Q9eBp3yGj"
SPLITTER_PUBLIC_KEY = "4HAz1eNba28njBhWKeVRUUn4tSobY1rNPP6MdUwMoBpa"

RARITY = ["Common", "Rare", "Epic", "Legendary"]

EQUIPMENT_TYPE = ["Head", "Staff", "Robe"]
EQUIPMENT_FEATURE_VALUE = ["Fire", "Earth", "Water"]
EQUIPMENT_FEATURE_PERCENT = ["Power", "Magic"]

SPELL_TYPE = ["Fire", "Earth", "Water", "Experience", "Craft", "Item"]

SPELL_MULTIPLE = 4

MULTIPLIERS = {
    "Common": 10,
    "Rare": 20,
    "Epic": 30,
    "Legendary": 40
}

PERCENT_ITEM_MULTIPLIERS = {
    "Common": 100,
    "Rare": 200,
    "Epic": 300,
    "Legendary": 400
}

RARITY_ODDS_SPELL = {
    "Common": {
        "min": 1,
        "max": 10,
        "odds": 8
    },
    "Rare": {
        "min": 11,
        "max": 20,
        "odds": 6
    },
    "Epic": {
        "min": 21,
        "max": 30,
        "odds": 4
    },
    "Legendary": {
        "min": 31,
        "max": 40,
        "odds": 2
    }
}

IMAGES = {
    "Chest": {
        1: "https://arweave.net/0hY6eGAP0MhUwe767vBNgyPGFCeHlh2hXLhDv6TuUxU",
        2: "https://arweave.net/4hU7ZdLhH4SGx-kqglfnvY24Prp4fJp1vLiJG-fJ4dE",
        3: "https://arweave.net/cM4gGA4oqFjz5V1kHjbibI9IbZ4Dm1CeiZrcS8MS1m4",
        4: "https://arweave.net/mgcSCC-E9EKpgIjbOtFwk3a9wM91rX59P_YGoVc7_X4"
    },
    "Robe": {
        "Common": {
            1: "https://arweave.net/jsu9Lxv7V8DL1sF-LnjK0CgwK2yWlmlBR8Xn7Qg9na8",
            2: "https://arweave.net/wdV-wPPgtXgkMhlOsmNy0vsC-VoOe5-z0eN0HpAfRl4",
            3: "https://arweave.net/tbQ7_qdpUz8zN_Spe6lsMX1O51hzNnbRqn45Pey_AVQ",
            4: "https://arweave.net/XKep6QEj3lHFde_5cj4-ckuc0GSKINU4qJyFf9p6Qig"
        },
        "Rare": {
            1: "https://arweave.net/MyFx2aisEdrPQh5RkgYRqKYGUvQCdXCfd5aMjtyIuKU",
            2: "https://arweave.net/F5tDEpDv45-Th_Y6rpO0mQwq9U95iy3FXeC8J94TXaY",
            3: "https://arweave.net/TBJIuy2YWav5fSUWOJk-UN40CB7jpgXAPzQlPLdMsbM",
            4: "https://arweave.net/nKV9GtEzxE-ybEj4XEXuVqtzmiN-K0G9EV4FJ19zLEM"
        },
        "Epic": {
            1: "https://arweave.net/7wWxBBkhhwB0QWmjcqXfxjdy8cP6AtZ7yIAn25vp1So",
            2: "https://arweave.net/OJSvUAX0uHq15pzBN3vJnzmj5fEgoLb5bhH7lbpwkkw",
            3: "https://arweave.net/aF_qClZYZGvXIxWNqwX0Qrb_LHmZiJ67-jxHD6RPJFE",
            4: "https://arweave.net/nWpU0sXT4gz4YCRwK43TcmpbmE_vqn46VRlOjvIzh8Y"
        },
        "Legendary": {
            1: "https://arweave.net/EWjNwYYqJOT7O0NKjxgMDGmPJlQceNDkcIFIpWEiK5M",
            2: "https://arweave.net/3dqyYm9Tdy6gUdmWkAFFy6zcVhaEKPO1bLHGfgrcurs",
            3: "https://arweave.net/bdyiWwhi3DKz68n_eu8U6Mfz7dpBOklfe3CLvaq9VLs",
            4: "https://arweave.net/2W9PV9k2hvWOPSrNIagz0-rU7WcK4op-lbEXBkNpnxs"
        }
    },
    "Staff": {
        "Common": {
            1: "https://arweave.net/_KRIUoC_mX4TIfnWL4q8cqFWPw-JRRcqfrVgEuLSV-c",
            2: "https://arweave.net/krNc1YQmpGP3ZwScwFrqE6TFRNm58PhCeWpl_3fvtTc",
            3: "https://arweave.net/-MdcE0KNmCWIZVaYPc0Nk2h_aeQ3jpsfLWuRaLqiTmM",
            4: "https://arweave.net/ncPZh_yN4fFW3R3jG2-paPRN-Uc7rEAPFXKFK-7UEKk"
        },
        "Rare": {
            1: "https://arweave.net/kxXwG3jWPOCZ96WauGXLqiOusBtQo7_f7cTM4D-owxY",
            2: "https://arweave.net/FsQ7V-aS91GudQpVFnKDee_XAI4MrZP983abrIytVs8",
            3: "https://arweave.net/j-ONW3b9a44f5KkfuoUArS7nEHQw-6fNEEXk_sZQhx0",
            4: "https://arweave.net/AhzjyeF4ezgQ8NQYy64WXN8l9TTNOfrAdSuCm1EpOco"
        },
        "Epic": {
            1: "https://arweave.net/Pn9DAmH1fZKkVjw3hK4fsd-pecyqmlmytiVgv1aD7-0",
            2: "https://arweave.net/kIkBmXoTrKtD5txTsOGR2JeG8r_-nJvukDvfFHszmXg",
            3: "https://arweave.net/zmc21Fyx-uS1qEf7DljjmokCLKpJJF5aeBRkkx4WiJ4",
            4: "https://arweave.net/uz7MkgPqWOs8E9dqIYNsuUu-YRQD1cK4BimdV2LVkfE"
        },
        "Legendary": {
            1: "https://arweave.net/dVfMQU4g8EuBtW769J2HUxrDMdKZPXKTUqAff6LogIc",
            2: "https://arweave.net/u_sN1lzCZbPNkHgX-DPXEMxRXnVzX6mtAEEjUwxMCRk",
            3: "https://arweave.net/AagbxFBATxIUvSlRH6bWgbZUcNMnFVKLnPVbiLAk934",
            4: "https://arweave.net/xtxc0pPLafQ5BxinGjW4dlXFpb_DS3-ELxYV5eVUGZo"
        }
    },
    "Head": {
        "Common": {
            1: "https://arweave.net/rmKbp3r4gmp7RSDikr6_rteoREvEFRd1EAFl7vLuHPI",
            2: "https://arweave.net/VuFLm6WWTY1wZa5UwVmrabRgDpWJiMJu44VAExcs6cs",
            3: "https://arweave.net/EuPQlGQZdFF11_5ggXuh_a1bOCO7tNjxRJ8RnyFG0vA",
            4: "https://arweave.net/KNPEHpHaOTG-revDOngTotUXkpshmHb1O-XvL6XeQ2o"
        },
        "Rare": {
            1: "https://arweave.net/V2dd6C48RS_D3AGyi1z5P9Oys-7n8lPmfe6asPoxMqo",
            2: "https://arweave.net/0nzBEXgbacM2XmK3wq5bfmuEs-3Fq2V87NeFuRUIr7o",
            3: "https://arweave.net/rafEnGXFqNsj0gJE07HRmFRX_iIn-mK5MgYZ6j3IrlQ",
            4: "https://arweave.net/WiNu9K_irAfnvM3Lvt3BLy1i6f36QgSvTHvAZjm645k"
        },
        "Epic": {
            1: "https://arweave.net/-RS6NUfl9QhpD--mIRSM3fb9DMIWrUqQKzfrk2rD1Zg",
            2: "https://arweave.net/KG9JMgcmpS6Pn9HTSQV_2ylhtffPSCulndACaJFojWI",
            3: "https://arweave.net/AjaLOxbJyxDszSRjC9m4gp2RsD0ZUCP3-03QI6lNP3s",
            4: "https://arweave.net/9jd7mzKFkmpdFVz86aHn-oMjYB6iRX6oth-Ay04zDB8"
        },
        "Legendary": {
            1: "https://arweave.net/SOOMVt6Ny1_Tw-2BeHqocVCs_1o9FOu2eMG_JcCyaOE",
            2: "https://arweave.net/UQcfwKQfq5qLV_IvbeUn7SluLRnc2VnWP5BpgVfh89M",
            3: "https://arweave.net/ZS3ivKw7sKb99FCG-FbBNOVwUZwb7hMB6Qqy8OomP34",
            4: "https://arweave.net/13MMSM8tuMLJTLWjNPLSZyTGIxAXJIsd8oAFR0Y9muk"
        }
    },
    "Spellbook": {
        "Common": {
            1: "https://arweave.net/SY_EiU6uizkqeq_onrgA1boA-o3n-_cyFQSzWp23jzo",
            2: "https://arweave.net/DUn44CZsiPNmudYRFZjH9iEhqUh6WR79hzUq88DV5qM",
            3: "https://arweave.net/N5AZFooZPlh-t5Kvana8NB9HWjVWZo-qknZ1RMfIIgY",
            4: "https://arweave.net/PLjTSZKhSmBUruAHyaM_-fv4WnBxeibIYy6KjBl2zu0"
        },
        "Rare": {
            1: "https://arweave.net/4GPei6IUP5hLIZQZuqiczxqL17MyAqpnMz7dhNZtQ2c",
            2: "https://arweave.net/i0pw5xEbm28tDD8rmHf1Ac90DThaLfm11OGBdgX6RZY",
            3: "https://arweave.net/G5FOO9gEPT33nHJfuIr4CYV3p-p6S_TlwjmLdGSYOlA",
            4: "https://arweave.net/lAiMNPLmNzgi--iybS6onM7CBuJgHuypLRGNvuFFXfU"
        },
        "Epic": {
            1: "https://arweave.net/gQ_FQGswSheNS5F3wu2hiFVlMF_WcIIa5wBSxwCt1TY",
            2: "https://arweave.net/s60SSiWPx-dsVZR7yVNNmmed9QAXXeaz6N6O7Jymd5s",
            3: "https://arweave.net/ER25mLMFXSX3Hefiu66JC6pvnhUcOzsMQqJ42XfgJac",
            4: "https://arweave.net/90Y6eQKvrdqVleV7w_s_uqcZye-KgLTtmoe1L4lKo3s"
        },
        "Legendary": {
            1: "https://arweave.net/ZEhqXzm-j23eE64cv2qv7l-Awyv9bhhroaqgBT42_10",
            2: "https://arweave.net/1f2b_SrvyM2d8Ic7PbwtqOdPiUYvSOlPcuRPGxI2KhI",
            3: "https://arweave.net/Z31hzD7vmCQeZI4fcJc4Vfi4N_Af-LA2oracF1TWeRk",
            4: "https://arweave.net/8ozs6H8uw2LFHb5RJgbsn_Y61aCPCibUqlhHDhBRh7M"
        }
    },
    "Caster": "https://arweave.net/F2TR14Rptj_LaS2iEd7t2FfUICVBccos8UEOPoBvEUo"
}

# JSON STANDARD for meta data info

JSON_TEMPLATE = {
    "name": "",
    "symbol": "LC",
    "description": "LadderCaster NFT",
    "seller_fee_basis_points": 100,
    "image": "",
    "external_url": "https://laddercaster.com",
    "attributes": [],
    "collection": {},
    "properties": {
        "files": [
            {
                "uri": "",
                "type": "image/png"
            }
        ],
        "creators": [
            {
                "address": CREATOR_PUBLIC_KEY,
                "verified": True,
                "share": 0
            },
            {
                "address": SPLITTER_PUBLIC_KEY,
                "verified": False,
                "share": 100
            }
        ]
    }
}

JSON_ATTRIBUTES_CHEST_TEMPLATE = [
    {
        "trait_type": "level",
        "value": ""
    },
    {
        "trait_type": "tier",
        "value": ""
    }
]

JSON_ATTRIBUTES_EQUIPMENT_TEMPLATE = [
    {
        "trait_type": "level",
        "value": ""
    },
    {
        "trait_type": "feature",
        "value": ""
    },
    {
        "trait_type": "rarity",
        "value": ""
    },
    {
        "trait_type": "type",
        "value": ""
    },
    {
        "trait_type": "value",
        "value": ""
    }
]

JSON_ATTRIBUTES_SPELLBOOK_TEMPLATE = [
    {
        "trait_type": "level",
        "value": "{}"
    },
    {
        "trait_type": "spell_type",
        "value": "{}"
    },
    {
        "trait_type": "cost",
        "value": "{}"
    },
    {
        "trait_type": "cost_feature",
        "value": "{}"
    },
    {
        "trait_type": "rarity",
        "value": "{}"
    },
    {
        "trait_type": "value",
        "value": "{}"
    }
]

JSON_ATTRIBUTES_CASTER_TEMPLATE = [
    {
        "trait_type": "level",
        "value": ""
    }
]


def _get_tier_based_on_level(equip_level):
    if equip_level <= 10:
        return 1

    if 11 <= equip_level <= 15:
        return 2

    if 16 <= equip_level <= 20:
        return 3

    if 21 <= equip_level <= 30:
        return 4


# Create JSON representations
def create_json_representation_chest(chest_level, chest_tier):
    base_template = copy.deepcopy(JSON_TEMPLATE)

    base_template["name"] = "Chest"
    base_template["image"] = IMAGES["Chest"][chest_tier]
    base_template["properties"]["files"][0]["uri"] = IMAGES["Chest"][chest_tier]

    chest_template = copy.deepcopy(JSON_ATTRIBUTES_CHEST_TEMPLATE)

    chest_template[0]["value"] = chest_level
    chest_template[1]["value"] = chest_tier

    base_template["attributes"].extend(chest_template)

    return base_template


def create_json_representation_equipment(equip_level, feature, equip_rarity, equip_type, equip_value):
    base_template = copy.deepcopy(JSON_TEMPLATE)

    base_template["name"] = equip_type
    base_template["image"] = IMAGES[equip_type][equip_rarity][
        _get_tier_based_on_level(equip_level)]
    base_template["properties"]["files"][0]["uri"] = IMAGES[equip_type][equip_rarity][
        _get_tier_based_on_level(equip_level)]

    equipment_template = copy.deepcopy(JSON_ATTRIBUTES_EQUIPMENT_TEMPLATE)

    equipment_template[0]["value"] = equip_level
    equipment_template[1]["value"] = feature
    equipment_template[2]["value"] = equip_rarity
    equipment_template[3]["value"] = equip_type
    equipment_template[4]["value"] = equip_value

    base_template["attributes"].extend(equipment_template)

    return base_template


def create_json_representation_spell_book(sp_level, spell_type, spell_cost, spell_cost_feature, spell_rarity,
                                          spell_value):
    base_template = copy.deepcopy(JSON_TEMPLATE)

    base_template["name"] = spell_type
    base_template["image"] = IMAGES["Spellbook"][spell_rarity][_get_tier_based_on_level(sp_level)]
    base_template["properties"]["files"][0]["uri"] = IMAGES["Spellbook"][spell_rarity][
        _get_tier_based_on_level(sp_level)]

    spell_template = copy.deepcopy(JSON_ATTRIBUTES_SPELLBOOK_TEMPLATE)

    spell_template[0]["value"] = sp_level
    spell_template[1]["value"] = spell_type
    spell_template[2]["value"] = spell_cost
    spell_template[3]["value"] = spell_cost_feature
    spell_template[4]["value"] = spell_rarity
    spell_template[5]["value"] = spell_value

    base_template["attributes"].extend(spell_template)

    return base_template


def create_json_representation_caster(caster_lvl):
    base_template = copy.deepcopy(JSON_TEMPLATE)

    base_template["name"] = "Caster"
    base_template["image"] = IMAGES["Caster"]
    base_template["properties"]["files"][0]["uri"] = IMAGES["Caster"]

    caster_template = copy.deepcopy(JSON_ATTRIBUTES_CASTER_TEMPLATE)

    caster_template[0]["value"] = caster_lvl

    base_template["attributes"].extend(caster_template)

    return base_template


# Chest
# for level in range(1, 6):
#     file_name = f"chest_{level}_{1}.json"
#     f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#     f.write(json.dumps(create_json_representation_chest(level, 1), indent=None, separators=(',', ':')))
#     f.close()
#
# for level in range(6, 11):
#     file_name = f"chest_{level}_{2}.json"
#     f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#     f.write(json.dumps(create_json_representation_chest(level, 2), indent=None, separators=(',', ':')))
#     f.close()
#
# for level in range(11, 16):
#     file_name = f"chest_{level}_{3}.json"
#     f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#     f.write(json.dumps(create_json_representation_chest(level, 3), indent=None, separators=(',', ':')))
#     f.close()
#
# for level in range(16, 31):
#     file_name = f"chest_{level}_{4}.json"
#     f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#     f.write(json.dumps(create_json_representation_chest(level, 4), indent=None, separators=(',', ':')))
#     f.close()

# # # Equipment
equipment_count = 0
for e_type in EQUIPMENT_TYPE:
    for level in range(1, 11):
        for rarity in RARITY:
            for percent_item in EQUIPMENT_FEATURE_PERCENT:
                for value in range(100,
                                   (round(level / 3 + 100) + PERCENT_ITEM_MULTIPLIERS[
                                       rarity]) + 1):  # + 1 on max since not inclusive

                    file_name = f"{e_type.lower()}_{level}_{percent_item.lower()}_{rarity.lower()}_{value}.json"
                    f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
                    f.write(json.dumps(
                        create_json_representation_equipment(level, percent_item, rarity,
                                                             e_type,
                                                             value),
                        indent=None, separators=(',', ':')))
                    f.close()
                    equipment_count += 1

            for value_item in EQUIPMENT_FEATURE_VALUE:
                for value in range(level * (MULTIPLIERS[rarity] - 10) + 1,
                                   level * MULTIPLIERS[rarity] + 1):  # + 1 on max since not inclusive

                    file_name = f"{e_type.lower()}_{level}_{value_item.lower()}_{rarity.lower()}_{value}.json"
                    f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
                    f.write(json.dumps(
                        create_json_representation_equipment(level, value_item, rarity, e_type,
                                                             value),
                        indent=None, separators=(',', ':')))
                    f.close()
                    equipment_count += 1

print("equipment count ", equipment_count)
#
# Spellbook
# spellbook_count = 0
# for s_type in SPELL_TYPE:
#     for level in range(1, 31):
#         for cost in (level, level + 10 + 1):  # + 1 on max since not inclusive
#             for rarity in RARITY:
#                 for cost_feature in ["Fire", "Water", "Earth"]:
#                     value = 0
#                     if s_type in ["Item", "Craft"]:
#                         file_name = f"spellbook_{level}_{s_type.lower()}_{cost_feature.lower()}_{rarity.lower()}_{cost}_{value}.json"
#                         f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#                         f.write(json.dumps(
#                             create_json_representation_spell_book(level, s_type, cost, cost_feature, rarity, value),
#                             indent=None, separators=(',', ':')))
#                         f.close()
#
#                         spellbook_count += 1
#
#                     if s_type in ["Experience"]:
#                         value = ((cost * (RARITY_ODDS_SPELL[rarity]["odds"] - 1)) + (SPELL_MULTIPLE * round(
#                             RARITY_ODDS_SPELL[rarity]["min"] * RARITY_ODDS_SPELL[rarity]["max"] / 3))) * 2
#
#                         file_name = f"spellbook_{level}_{s_type.lower()}_{cost_feature.lower()}_{rarity.lower()}_{cost}_{value}.json"
#                         f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#                         f.write(json.dumps(
#                             create_json_representation_spell_book(level, s_type, cost, cost_feature, rarity, value),
#                             indent=None, separators=(',', ':')))
#                         f.close()
#
#                         spellbook_count += 1
#
#                     if s_type in ["Fire", "Earth", "Water"]:
#                         value = ((cost * (RARITY_ODDS_SPELL[rarity]["odds"] - 1)) + (SPELL_MULTIPLE * round(
#                             RARITY_ODDS_SPELL[rarity]["min"] * RARITY_ODDS_SPELL[rarity]["max"] / 3))) * 1
#
#                         file_name = f"spellbook_{level}_{s_type.lower()}_{cost_feature.lower()}_{rarity.lower()}_{cost}_{value}.json"
#                         f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#                         f.write(json.dumps(
#                             create_json_representation_spell_book(level, s_type, cost, cost_feature, rarity, value),
#                             indent=None, separators=(',', ':')))
#                         f.close()
#
#                         spellbook_count += 1

# print(spellbook_count)
#
# # Casters
# for i in range(1, 31):
#     file_name = f"caster_1_{i}.json"
#     f = open(os.path.dirname(__file__) + f'/arweave/json_exports/{file_name}', 'w')
#     f.write(json.dumps(create_json_representation_caster(i), indent=None, separators=(',', ':')))
#     f.close()
