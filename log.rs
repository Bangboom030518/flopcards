Button {
    element: GenericElement {
        tag_name: "button",
        attributes: Attributes(
            {
                "class": "w-20 h-20 bg-pink-600 shadow-md fixed bottom-0 transition right-0 hover:shadow-lg hover:bg-pink-700 focus:bg-pink-700 m-8 active:bg-pink-800 border-8 border-pink-700 rounded-full",
            },
        ),
        children: [
            VoidElement(
                VoidElement {
                    tag_name: "img",
                    attributes: Attributes(
                        {
                            "alt": "Create New",
                            "class": "w-full h-full m-0",
                            "src": "/assets/create.svg",
                        },
                    ),
                },
            ),
            Element(
                GenericElement {
                    tag_na2:48 Pme: "p",
                    attributes: Attributes(
                        {},
                    ),
                    children: [
                        Text(
                            "Create New",
                        ),
                    ],
                },
            ),
            Element(
                GenericElement {
                    tag_name: "button",
                    attributes: Attributes(
                        {},
                    ),
                    children: [
                        Element(
                            GenericElement {
                                tag_name: "button",
                                attributes: Attributes(
                                    {},
                                ),
                                children: [
                                    Text(
                                        "Import Audio",
                                    ),
                                ],
                            },
                        ),
                        Element(
                            GenericElement {
                                tag_name: "button",
                                attributes: Attributes(
                                    {},
                                ),
                                children: [
                                    Text(
                                        "Create Remix",
                                    ),
                                ],
                            },
                        ),
                    ],
                },
            ),
        ],
    },
}
