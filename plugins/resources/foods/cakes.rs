#[allow(dead_code)]
struct Cake {
    name: &'static str,
    file: &'static str,
    location: &'static str,
    info: &'static str,
}

static CAKES: &'static [Cake] = &[
    Cake {
        name: r#"Angel cake"#,
        file: r#"Angel cake slice.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Sponge cake, cream"#,
    },
    Cake {
        name: r#"Angel food cake"#,
        file: r#"AngelFoodCake.jpg"#,
        location: r#"United States"#,
        info: r#"Egg whites, vanilla and Potassium bitartrate|cream of tartar"#,
    },
    Cake {
        name: r#"Apple cake"#,
        file: r#"Hollandse appeltaart.jpg"#,
        location: r#"Unknown"#,
        info: r#"Apple, caramel icing"#,
    },
    Cake {
        name: r#"Aranygaluska"#,
        file: r#"Arany-galuska.jpg"#,
        location: r#"Hungary, Romania"#,
        info: r#"A cake with yeasty dough and vanilla custard"#,
    },
    Cake {
        name: r#"Babka (cake)|Babka"#,
        file: r#"Baba or babka wielkanocna.jpg"#,
        location: r#"Poland"#,
        info: r#"Easter Cake with icing"#,
    },
    Cake {
        name: r#"Ballokume"#,
        file: r#""#,
        location: r#"Albania"#,
        info: r#"Corn flour, butter, sugar and vanilla"#,
    },
    Cake {
        name: r#"Basbousa"#,
        file: r#"Basboosa.jpg"#,
        location: r#"Somalia"#,
        info: r#"A traditional Somali sweet cake that is made of cooked semolina or farina soaked in simple syrup. Coconut is a popular addition. The syrup may also optionally contain orange flower water or rose water."#,
    },
    Cake {
        name: r#"Better than sex cake"#,
        file: r#""#,
        location: r#"United States"#,
        info: r#"Chocolate or yellow cake, sugar mixture, various fillings"#,
    },
    Cake {
        name: r#"Boston cream pie"#,
        file: r#"Bostoncreampie.jpg"#,
        location: r#"United States"#,
        info: r#"Egg custard, chocolate"#,
    },
    Cake {
        name: r#"Banana bread|Banana cake/bread"#,
        file: r#"Banananutbread.jpg"#,
        location: r#"United States"#,
        info: r#"Banana, sometimes nuts and chocolate"#,
    },
    Cake {
        name: r#"Banoffee pie"#,
        file: r#"Banoffeepie.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Bananas, toffee, biscuits"#,
    },
    Cake {
        name: r#"Bara brith"#,
        file: r#"Bara Brith.jpg"#,
        location: r#"United Kingdom (Wales)"#,
        info: r#"Raisins, Ribes|currants and candied peel"#,
    },
    Cake {
        name: r#"Battenberg cake"#,
        file: r#"Lyons battenberg cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Marzipan and apricot jam"#,
    },
    Cake {
        name: r#"Baumkuchen"#,
        file: r#"Baumkuchen.jpg"#,
        location: r#"Germany"#,
        info: r#"A kind of layered cake and a traditional dessert that is in many countries throughout Europe and it is also a popular snack and dessert in Japan. The characteristic rings that appear when sliced resemble tree rings, and give the cake its German name, Baumkuchen, which literally translates to "tree cake"."#,
    },
    Cake {
        name: r#"Bibingka"#,
        file: r#"Large bibinka.jpg"#,
        location: r#"Philippines"#,
        info: r#"Coconut milk and rice flour"#,
    },
    Cake {
        name: r#"Bienenstich (Bee Sting)"#,
        file: r#"Bienenstich.jpg"#,
        location: r#"Germany"#,
        info: r#"Almonds, honey, custard cream"#,
    },
    Cake {
        name: r#"Birthday cake"#,
        file: r#"Birthday cake.jpg"#,
        location: r#"Worldwide"#,
        info: r#"A cake that has various ingredients, usually chocolate or sponge, and is often topped with Icing (food)|icing and candles; number of candles on top of the cake is often said to represent some one's age, for example, a birthday cake for a nine year old will have nine candles on top of it."#,
    },
    Cake {
        name: r#"Black Forest cake, often known as "Black Forest gâteau""#,
        file: r#"Kirschtorte.jpg"#,
        location: r#"Germany"#,
        info: r#"Cherries, Kirschwasser|kirsch, and chocolate"#,
    },
    Cake {
        name: r#"Blondie (confection)|Blondie"#,
        file: r#"Toffee blondies.jpg"#,
        location: r#"United States"#,
        info: r#"A rich, sweet dessert bar. It is made from flour, brown sugar, butter, eggs, baking powder, and vanilla, and may also contain walnuts or pecans. It may contain white or dark chocolate chips and it can have a taste reminiscent of butterscotch."#,
    },
    Cake {
        name: r#"Chocolate brownie|Brownie"#,
        file: r#"Chocolate brownie.jpg"#,
        location: r#"United States, Canada"#,
        info: r#"A flat, baked square or bar developed in the United States at the end of the 19th century and popularized in both the U.S. and Canada during the first half of the 20th century."#,
    },
    Cake {
        name: r#"Buccellato"#,
        file: r#"Buccellato 1.jpg"#,
        location: r#"Sicily"#,
        info: r#"Honey, marsala, aniseed, and raisins"#,
    },
    Cake {
        name: r#"Budapestlängd"#,
        file: r#"Budapestbakelse BÅn.JPG"#,
        location: r#"Sweden"#,
        info: r#"Rolled meringue-hazelnut cake filled with whipped cream and pieces of tin can|canned peach, apricot or mandarin orange."#,
    },
    Cake {
        name: r#"Bundt cake"#,
        file: r#"BundtCake.JPG"#,
        location: r#"United States"#,
        info: r#"A cake that is baked in a Bundt pan, shaping it into a distinctive ring shape. The shape is inspired by a traditional European fruit cake known as Gugelhupf, but Bundt cakes are not generally associated with any single recipe, but they are often made with chocolate."#,
    },
    Cake {
        name: r#"Butter cake"#,
        file: r#"Hazelnut brown butter cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Butter"#,
    },
    Cake {
        name: r#"Butterfly cake"#,
        file: r#"Plain butterfly cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"A variant of cupcake, also called fairy cake for its fairy-like "wings". They can be made from any flavor of cake. The top of the fairy cake is cut off or carved out with a spoon, and cut in half. Then, butter cream, whipped cream or other sweet filling like jam is spread into the hole. Finally, the two cut halves are stuck into the butter cream to look like butterfly wings. The wings of the cake are often decorated using icing to form various patterns."#,
    },
    Cake {
        name: r#"Carrot cake"#,
        file: r#"Carrot cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Carrots"#,
    },
    Cake {
        name: r#"Cheesecake (cake)|Cheesecake"#,
        file: r#""#,
        location: r#"Greece"#,
        info: r#"Thin base made from crushed biscuits, with a thicker top layer of soft cheese, eggs and sugar. It can be baked or unbaked (in which case it is refrigerated.)"#,
    },
    Cake {
        name: r#"Chiffon cake"#,
        file: r#"Chiffon cake 02.jpg"#,
        location: r#"United States"#,
        info: r#"Vegetable oil, eggs, sugar, flour"#,
    },
    Cake {
        name: r#"Chocolate cake"#,
        file: r#"Chocolate cake with chocolate frosting topped with chocolate.jpg"#,
        location: r#"Unknown"#,
        info: r#"Chocolate"#,
    },
    Cake {
        name: r#"Christmas cake"#,
        file: r#"Christmas cake, Boxing Day 2008.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Dried fruit such as sultana (grape)|sultanas or raisins; cinnamon, treacle, cherries, and almond; is quite often topped with Icing (food)|icing. If topped with icing, the icing may be decorated with decorations, such as models of Santa Claus, or may have labels such as "Happy Christmas"."#,
    },
    Cake {
        name: r#"Cinco leches cake"#,
        file: r#""#,
        location: r#"Mexico"#,
        info: r#"A cake that consists of five milks."#,
    },
    Cake {
        name: r#"Coconut cake"#,
        file: r#""#,
        location: r#"United States"#,
        info: r#"A popular dessert in the Southern region of the United States. It is a cake frosted with a white frosting and covered in coconut flakes."#,
    },
    Cake {
        name: r#"Coffee cake"#,
        file: r#"Walnut cinnamon coffee cake.jpg"#,
        location: r#"Germany"#,
        info: r#"Cinnamon"#,
    },
    Cake {
        name: r#"Cremeschnitte"#,
        file: r#"Kremna rezina.jpg"#,
        location: r#"Slovenia, Croatia, Germany"#,
        info: r#"A vanilla and custard cream cake dessert popular in several central-European countries. There are many regional variations, but they all include puff pastry base and custard cream."#,
    },
    Cake {
        name: r#"Croquembouche"#,
        file: r#"Croquembouche wedding cake.jpg"#,
        location: r#"France"#,
        info: r#"Caramel, almond, and chocolate"#,
    },
    Cake {
        name: r#"Crystal cake"#,
        file: r#"Crystal cake.jpg"#,
        location: r#"China"#,
        info: r#"One of the traditional desserts in China. It has more than 800 years of history. It was first invented in Xiagui during the Song Dynasty, then it spread far and wide. People named it as Crystal Cake, because its filling shines brightly, and its appearance is glittering and translucent, like a crystal."#,
    },
    Cake {
        name: r#"Cuatro leches cake"#,
        file: r#""#,
        location: r#"Spain, Mexico"#,
        info: r#"A cake that consists of four milks and it is similar to the tres leches cake."#,
    },
    Cake {
        name: r#"Cupcake"#,
        file: r#"Chocolate cupcakes.jpg"#,
        location: r#"Worldwide"#,
        info: r#"A small cake with various ingredients, usually topped with icing"#,
    },
    Cake {
        name: r#"Dacquoise"#,
        file: r#"Eggnog mousse cake with almond dacquoise.jpg"#,
        location: r#"France"#,
        info: r#"Almonds, hazelnut, and chocolate"#,
    },
    Cake {
        name: r#"Date and walnut loaf"#,
        file: r#"Date and walnut bread.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Phoenix dactylifera|Dates, walnuts, Molasses|treacle, and tea"#,
    },
    Cake {
        name: r#"Date square"#,
        file: r#"Date squares.jpg"#,
        location: r#"Canada (probably)"#,
        info: r#"Also known as Matrimonial Cake, is a layer of minced dates with oat crumble"#,
    },
    Cake {
        name: r#"Depression cake"#,
        file: r#""#,
        location: r#"United States"#,
        info: r#"Made without milk, sugar, butter, or eggs"#,
    },
    Cake {
        name: r#"Devil's food cake"#,
        file: r#"Devil's Food Cake.jpg"#,
        location: r#"United States"#,
        info: r#"Chocolate and/or Cocoa solids|cocoa, and baking soda"#,
    },
    Cake {
        name: r#"Dobos cake"#,
        file: r#"Dobos cake (Gerbeaud Confectionery Budapest Hungary).jpg"#,
        location: r#"Hungary"#,
        info: r#"A sponge cake that is layered with chocolate butter cream and topped with thin caramel slices"#,
    },
    Cake {
        name: r#"Dundee cake"#,
        file: r#"Dundee_cake.jpg"#,
        location: r#"United Kingdom (Scotland)"#,
        info: r#"Fruit cake with almonds on it but without glace cherries."#,
    },
    Cake {
        name: r#"Eccles cake"#,
        file: r#"Eccles cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Zante currants"#,
    },
    Cake {
        name: r#"Esterházy torte"#,
        file: r#"Eszterházy Torte.JPG"#,
        location: r#"Hungary, Austria"#,
        info: r#"A Hungarian cake (torta) named after Prince Paul III Anton Esterházy de Galántha (1786–1866). It was invented by Budapest confectioners in the late 19th century. It consists of cognac or vanilla buttercream, sandwiched between layers of almond meringue (macaroon) dough. The torte is iced with a fondant glaze and decorated with a characteristic chocolate striped pattern."#,
    },
    Cake {
        name: r#"Fat rascal"#,
        file: r#"Fat Rascal cookies.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Dried fruit, peel, oats"#,
    },
    Cake {
        name: r#"Faworki"#,
        file: r#"Faworkib.jpg"#,
        location: r#"Poland"#,
        info: r#"Sweet crisp cake in shape of a bow"#,
    },
    Cake {
        name: r#"Financier (cake)|Financier"#,
        file: r#"Two rectangular financiers.jpg"#,
        location: r#"France"#,
        info: r#"A small French cake. The financier is light and moist, similar to sponge cake, and usually contains almond flour, crushed or ground almonds, or almond flavoring. The distinctive feature of the recipe is beurre noisette (brown butter). Other ingredients include egg whites, flour, and powdered sugar. They are baked in shaped molds, usually small rectangular loaves similar in size to petits fours. In terms of texture, it is springy with a crisp, eggshell-like exterior."#,
    },
    Cake {
        name: r#"Flourless chocolate cake"#,
        file: r#"Flourless Chocolate Cake with Bourbon Vanilla Ice Cream.jpg"#,
        location: r#"Unknown"#,
        info: r#"Chocolate"#,
    },
    Cake {
        name: r#"Fondant Fancy"#,
        file: r#"White Mr Kipling French fancy cake (82524785).jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Icing (food)|Icing (any of a number of colors), cream"#,
    },
    Cake {
        name: r#"Fragelité"#,
        file: r#"Konditor kager 1.JPG"#,
        location: r#"Denmark"#,
        info: r#"Meringue, almonds, butter, coffee"#,
    },
    Cake {
        name: r#"Frog cake"#,
        file: r#"Frog cakes.jpg"#,
        location: r#"Australia"#,
        info: r#"Cream, icing"#,
    },
    Cake {
        name: r#"Fruitcake"#,
        file: r#"Traditional fruitcake.jpg"#,
        location: r#"Ancient Rome"#,
        info: r#"Candied fruit; many versions of the fruit cake contain currants, sultanas and glace cherries."#,
    },
    Cake {
        name: r#"Funing big cake"#,
        file: r#""#,
        location: r#"China (Funing County, Jiangsu|Funing County, Jiangsu province)"#,
        info: r#"Sticky rice, white sugar and refined lard. Due to health concerns associated with lard consumption, sometimes vegetable oil is used instead of lard."#,
    },
    Cake {
        name: r#"Genoa cake"#,
        file: r#"Genoa-cake.jpg"#,
        location: r#"Italy (Genoa, probably)"#,
        info: r#"Sultanas, raisins, glacé cherries"#,
    },
    Cake {
        name: r#"Genoise (Genoese cake)"#,
        file: r#"Génoise cake with buttercream frosting.jpg"#,
        location: r#"Italy (Genoa, probably)"#,
        info: r#"Whole egg (food)|egg"#,
    },
    Cake {
        name: r#"Gingerbread"#,
        file: r#"Cakegingerbread.jpg"#,
        location: r#"United Kingdom (probably)"#,
        info: r#"Ginger"#,
    },
    Cake {
        name: r#"Gooey butter cake"#,
        file: r#"Gooey Pumpkin Butter Cake.jpg"#,
        location: r#"United States"#,
        info: r#"Butter"#,
    },
    Cake {
        name: r#"Goose Breast"#,
        file: r#"Flickr - cyclonebill - Gåsebryst.jpg"#,
        location: r#"Denmark"#,
        info: r#"A cream cake known as ''Gåsebryst'' in Denmark. A Danish pastry bottom, topped with whipped cream, custard and jam, wrapped in marzipan."#,
    },
    Cake {
        name: r#"Hash brownies"#,
        file: r#"KCCS Cookie.JPG"#,
        location: r#"Netherlands, Belgium"#,
        info: r#"Also known as space cakes, are bakery products made using one of the forms of cannabis, including hashish."#,
    },
    Cake {
        name: r#"Hot milk cake"#,
        file: r#""#,
        location: r#"United States (probably)"#,
        info: r#"Milk, and Cafe mocha|mocha"#,
    },
    Cake {
        name: r#"Ice cream cake"#,
        file: r#"Three inch ice cream cake with fruit from Singapore.jpg"#,
        location: r#"Unknown"#,
        info: r#"Ice cream"#,
    },
    Cake {
        name: r#"Jaffa Cakes"#,
        file: r#"Jaffa cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"A biscuit-sized cake introduced by MacVitie and Price in 1927 and named after Jaffa oranges. The most common form of Jaffa Cakes are circular, 2 1⁄2 inches (64&nbsp;mm) in diameter and have three layers: a Genoise sponge base, a layer of orange flavored jelly and a coating of chocolate."#,
    },
    Cake {
        name: r#"Kabuni"#,
        file: r#""#,
        location: r#"Albania"#,
        info: r#"Rice, butter, mutton broth, raisins, sugar, cinnamon, cloves"#,
    },
    Cake {
        name: r#"Karpatka"#,
        file: r#"Karpatka.jpg"#,
        location: r#"Poland"#,
        info: r#"Two to eight layers of very flattened sweet bake pastry with cream and sweet cheese, normally served with fruit and budyn  and cardamom and ice cream which may have alcohol also on the side of this luxurious dessert"#,
    },
    Cake {
        name: r#"Kiev cake"#,
        file: r#"Київський торт без коробки.JPG"#,
        location: r#"Ukraine"#,
        info: r#"Two airy layers of meringue with hazelnuts, chocolate glaze, and a buttercream-like filling"#,
    },
    Cake {
        name: r#"King cake"#,
        file: r#"Kingcake.jpg"#,
        location: r#"France, Spain"#,
        info: r#"Sugar, cinnamon, milk, and butter"#,
    },
    Cake {
        name: r#"Kladdkaka"#,
        file: r#"Kaffereptarta.jpg"#,
        location: r#"Sweden"#,
        info: r#"Chocolate"#,
    },
    Cake {
        name: r#"Kliņģeris"#,
        file: r#""#,
        location: r#"Latvia"#,
        info: r#"Yeast, raisins, spices"#,
    },
    Cake {
        name: r#"Kolacz"#,
        file: r#"Kolacz (cake).JPG"#,
        location: r#"Poland"#,
        info: r#"Sweet cheese and cream"#,
    },
    Cake {
        name: r#"Kolaczki"#,
        file: r#"Kolaczki - A Polish Pastry Type Of Cake.jpg"#,
        location: r#"Poland"#,
        info: r#"Butter, sugar, jam, egg whites, different sweet sugar powder"#,
    },
    Cake {
        name: r#"Kouign-amann"#,
        file: r#"Kouignamann.JPG"#,
        location: r#"France (Brittany)"#,
        info: r#"Butter"#,
    },
    Cake {
        name: r#"Kutia"#,
        file: r#"Koljivo from wheat.jpg"#,
        location: r#"Poland, Belarus, Ukraine, Lithuania, Russia"#,
        info: r#"Various nuts and raisins"#,
    },
    Cake {
        name: r#"Kransekake"#,
        file: r#"Kransekake.jpg"#,
        location: r#"Denmark, Norway"#,
        info: r#"Almonds, sugar, egg whites"#,
    },
    Cake {
        name: r#"Kremówka"#,
        file: r#"Kremowki dwie.JPG"#,
        location: r#"Germany, Slovakia"#,
        info: r#"A Polish type of cream pie. It is made of two layers of puff pastry, filled with whipped cream, creamy buttercream, vanilla pastry cream (custard cream) or sometimes egg white cream, and is usually sprinkled with powdered sugar. It also can be decorated with cream or covered with a layer of icing."#,
    },
    Cake {
        name: r#"Krówki|Krowka"#,
        file: r#"Boza krowka.jpg"#,
        location: r#"Poland"#,
        info: r#"Chocolate, sponge base, caramel and coconut"#,
    },
    Cake {
        name: r#"Lady Baltimore Cake"#,
        file: r#""#,
        location: r#"United States"#,
        info: r#"Dried fruit, nuts, frosting"#,
    },
    Cake {
        name: r#"Lamanki or Klamäti"#,
        file: r#"Łamańce.JPG"#,
        location: r#"Poland"#,
        info: r#"Chocolate, cinnamon"#,
    },
    Cake {
        name: r#"Lamington"#,
        file: r#"Lamington.png"#,
        location: r#"Australia"#,
        info: r#"Chocolate icing, and desiccated coconut"#,
    },
    Cake {
        name: r#"Layer cake"#,
        file: r#"Meyer lemon chiffon cake, chocolate.jpg"#,
        location: r#"Worldwide"#,
        info: r#"Yolk, sugar, butter, flour"#,
    },
    Cake {
        name: r#"Lemon cake"#,
        file: r#""#,
        location: r#"Unknown"#,
        info: r#"Lemon"#,
    },
    Cake {
        name: r#"Madeira cake"#,
        file: r#"Cherry madeira cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Butter and sugar"#,
    },
    Cake {
        name: r#"Poppy seed roll|Makowiec"#,
        file: r#"MakowiecCakb.jpg"#,
        location: r#"Poland"#,
        info: r#"Poppy seed cake normally decorated with icing and orange"#,
    },
    Cake {
        name: r#"Muffin|Magdalena"#,
        file: r#"Madalenas caseras-Madrid.jpg"#,
        location: r#"Spain"#,
        info: r#"Eggs, granulated sugar, unsalted butter, unbleached white flour, lemon zest, baking powder and milk"#,
    },
    Cake {
        name: r#"Mantecada"#,
        file: r#"Mantecadas de Tuesta-Valdegovía8.JPG"#,
        location: r#"Spain"#,
        info: r#"Eggs, flour, sugar and butter (cow fat in the Mantecadas de Astorga; Cornmeal|corn flour in Colombia)"#,
    },
    Cake {
        name: r#"Marble cake"#,
        file: r#"Marmorkuchen.jpg"#,
        location: r#"Denmark"#,
        info: r#"Vanilla, coffee, and/or chocolate"#,
    },
    Cake {
        name: r#"Mazurek (cake)|Mazurek"#,
        file: r#"Mazurek 1.JPG"#,
        location: r#"Poland"#,
        info: r#"Easter cake with a type of shortcrust tart and topping"#,
    },
    Cake {
        name: r#"Mille-feuille"#,
        file: r#"Mille-feuille français 1.jpg"#,
        location: r#"France"#,
        info: r#"Also known as a Napoleon, is three layers of puff pastry alternating with two layers of pastry cream. The top is glazed in white (icing) and brown (chocolate) strips, and combed."#,
    },
    Cake {
        name: r#"Molten chocolate cake"#,
        file: r#"Chocolate Fondant.jpg"#,
        location: r#"United States"#,
        info: r#"Also known as lava cake is a popular dessert that combines the elements of a flourless chocolate cake (sometimes called a chocolate decadence cake) and a soufflé. Some other names used are chocolate fondant, chocolate moelleux and chocolate lava cake."#,
    },
    Cake {
        name: r#"Mooncake"#,
        file: r#"Mooncake1.jpg"#,
        location: r#"China"#,
        info: r#"A Chinese bakery product traditionally eaten during the Mid-Autumn Festival (Zhongqiujie)."#,
    },
    Cake {
        name: r#"Muffin"#,
        file: r#"Muffins in oven.jpg"#,
        location: r#"Unknown"#,
        info: r#"An individual sized quick bread product which can be sweet or savory. The typical American muffin is similar to a cupcake in size and cooking methods. These can come in both savory varieties, such as corn or cheese muffins, or sweet varieties such as blueberry or banana. It also refers to a flatter disk-shaped bread of English origin, commonly referred to as an English muffin outside the United Kingdom. These muffins are also popular in Commonwealth countries and the United States."#,
    },
    Cake {
        name: r#"Napoleonshat"#,
        file: r#"Napoleonshat.jpg"#,
        location: r#"Denmark"#,
        info: r#"A marzipan based cake, shaped like a Napoleon's Hat and dipped in dark chocolate."#,
    },
    Cake {
        name: r#"Napeleonskake"#,
        file: r#""#,
        location: r#"Norway, Denmark, Iceland"#,
        info: r#"A cake that is similar to tompouce, but it has different flavors like caramel or carob."#,
    },
    Cake {
        name: r#"Nasturtium cake"#,
        file: r#""#,
        location: r#"Spain"#,
        info: r#"A cake made primarily with egg yolk and syrup. The cake is usually presented in a cylindrical shape or a rectangle, depending on the mold. This is a cake that is made by a water bath. It can often be served at room temperature."#,
    },
    Cake {
        name: r#"Oponki or Paczki"#,
        file: r#"Tisto.jpg"#,
        location: r#"Poland"#,
        info: r#"Round spongy yeast cake with sweet topping and other chocolate"#,
    },
    Cake {
        name: r#"Opera cake"#,
        file: r#"Tartine_bakery_opera_cake_in_2007.jpg"#,
        location: r#"France"#,
        info: r#"Ganache, sponge cake, and coffee syrup"#,
    },
    Cake {
        name: r#"Orange and polenta cake"#,
        file: r#""#,
        location: r#"Italy"#,
        info: r#"Oranges and polenta"#,
    },
    Cake {
        name: r#"Othellolagkage"#,
        file: r#"Othello-lagkager.jpg"#,
        location: r#"Denmark"#,
        info: r#"A layer cake with sponge cake, cream, chocolate, raspberry, egg, vanilla, marzipan,"#,
    },
    Cake {
        name: r#"Paczki"#,
        file: r#"Polish paczki.jpg"#,
        location: r#"Poland"#,
        info: r#"Round cake with spongy yeast containing strawberry, liqueur, budyn and ( sweet ) cheese and other chocolate"#,
    },
    Cake {
        name: r#"Pancake"#,
        file: r#"Blueberry pancakes (1).jpg"#,
        location: r#"Ancient Rome"#,
        info: r#"Flat, round cake, made with Egg_(food)|eggs, milk, and Flour|plain flour"#,
    },
    Cake {
        name: r#"Panpepato"#,
        file: r#"Panpepato-small.jpg"#,
        location: r#"Italy"#,
        info: r#"Varies, almonds, hazelnuts, pine nuts"#,
    },
    Cake {
        name: r#"Panettone"#,
        file: r#"Panettone.jpg"#,
        location: r#"Italy"#,
        info: r#"Raisins, orange peel, and lemon peel"#,
    },
    Cake {
        name: r#"Parkin (cake)|Parkin"#,
        file: r#"Darkparkin.JPG"#,
        location: r#"United Kingdom"#,
        info: r#"Treacle and oats"#,
    },
    Cake {
        name: r#"Pavlova (food)|Pavlova"#,
        file: r#"Christmas pavlova.jpg"#,
        location: r#"New Zealand"#,
        info: r#"Egg white and sugar (meringue); named after Anna Pavlova"#,
    },
    Cake {
        name: r#"Petit Gâteau"#,
        file: r#"Gastro petit-gateau-delicia.jpeg"#,
        location: r#"France"#,
        info: r#"Chocolate and served with Ice-cream"#,
    },
    Cake {
        name: r#"Petits fours"#,
        file: r#"Petits.fours.wmt.jpg"#,
        location: r#"France"#,
        info: r#"Butter cream"#,
    },
    Cake {
        name: r#"Piernik"#,
        file: r#"PiernikCakeb.jpg"#,
        location: r#"Poland"#,
        info: r#"Gingerbread with cinnamon, ginger, cloves and cardamom"#,
    },
    Cake {
        name: r#"Pineapple upside-down cake"#,
        file: r#"Pineapple upsidedown cake 9.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Pineapple"#,
    },
    Cake {
        name: r#"Pound cake"#,
        file: r#"Pound layer cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Butter, sugar"#,
    },
    Cake {
        name: r#"Princess Cake"#,
        file: r#"Prinsesstårta.JPG"#,
        location: r#"Sweden"#,
        info: r#"Alternating layers of sponge cake and whipped cream, a layer of berry jam and a layer of custard, all topped with a layer of (green) marzipan."#,
    },
    Cake {
        name: r#"Prinzregententorte"#,
        file: r#"Prinzregententorte.jpg"#,
        location: r#"Germany"#,
        info: r#"Sponge cake, buttercream and dark chocolate glaze"#,
    },
    Cake {
        name: r#"Pumpkin bread"#,
        file: r#"Pumpkin bread.jpg"#,
        location: r#"United States"#,
        info: r#"Pumpkin, sometimes chocolate"#,
    },
    Cake {
        name: r#"Punschkrapfen"#,
        file: r#"Image-Punschkrapfen.jpg"#,
        location: r#"Austria"#,
        info: r#"Cake crumbs, nougat chocolate, apricot jam, and rum"#,
    },
    Cake {
        name: r#"Queen Elizabeth cake"#,
        file: r#""#,
        location: r#"Canada (Quebec)"#,
        info: r#"Coconut, Phoenix dactylifera|dates"#,
    },
    Cake {
        name: r#"Qumeshtore me pete"#,
        file: r#""#,
        location: r#"Albania"#,
        info: r#"Cream, vanilla, syrup, lemon juice, and lemon skin"#,
    },
    Cake {
        name: r#"Red bean cake"#,
        file: r#"RedbeanCake Cantoversion.jpg"#,
        location: r#"Japan, China"#,
        info: r#"Azuki bean and red bean paste"#,
    },
    Cake {
        name: r#"Red velvet cake"#,
        file: r#"Red velvet cake slice.jpg"#,
        location: r#"United States"#,
        info: r#"Red coloring and cocoa"#,
    },
    Cake {
        name: r#"Rum cake"#,
        file: r#"Rum cake.jpg"#,
        location: r#"Jamaica, Trinidad and Tobago"#,
        info: r#"Rum, dried fruit"#,
    },
    Cake {
        name: r#"Rum baba"#,
        file: r#"Rum baba.jpg"#,
        location: r#"Italy|Italian"#,
        info: r#"Rum, yeast, whipped cream"#,
    },
    Cake {
        name: r#"Ruske kape"#,
        file: r#"RuskeKapa.jpg"#,
        location: r#"Bosnia, Serbia"#,
        info: r#"Chocolate, and coconut"#,
    },
    Cake {
        name: r#"Sachertorte"#,
        file: r#"Sachertorte DSC03027 retouched.jpg"#,
        location: r#"Austria"#,
        info: r#"Apricot, and cream"#,
    },
    Cake {
        name: r#"Šakotis"#,
        file: r#"Šakotis 3799.jpg"#,
        location: r#"Lithuania, Poland"#,
        info: r#"Traditional cake created by painting layers of dough onto a rotating spit while being baked."#,
    },
    Cake {
        name: r#"Salzburger Nockerl"#,
        file: r#"Salzburger Nockerln 04 gastronomie 001.jpg"#,
        location: r#"Austria"#,
        info: r#"Egg yolk, flour and milk"#,
    },
    Cake {
        name: r#"Sandwich loaf"#,
        file: r#"SandwichLoaf4.jpg"#,
        location: r#"United States"#,
        info: r#"A stacked party entrée that looks like a cake. While rare today, the food was quite popular during the mid 20th century in the United States. In order to create a sandwich loaf, the bread is cut horizontally and spread with layers of filling. Common fillings include egg salad, chicken salad, ham salad, tuna salad, and Cheez Whiz, but other fillings are possible, including peanut butter and jelly and mock egg salad made from tofu."#,
    },
    Cake {
        name: r#"Seis leches cake"#,
        file: r#""#,
        location: r#"Mexico, Spain"#,
        info: r#"A cake that was invented with six milks."#,
    },
    Cake {
        name: r#"Sekacz"#,
        file: r#"Sekacz (1).jpg"#,
        location: r#"Poland"#,
        info: r#"Sponge cake with chocolate"#,
    },
    Cake {
        name: r#"Sernik"#,
        file: r#"PolskiSernik.jpg"#,
        location: r#"Poland"#,
        info: r#"Cream cheese, sponge cake, raisins and different spices"#,
    },
    Cake {
        name: r#"Sesame seed cake"#,
        file: r#"Sesame seed cakes.jpg"#,
        location: r#"Worldwide"#,
        info: r#"Sesame seeds, often with honey as a sweetener"#,
    },
    Cake {
        name: r#"Sfouf"#,
        file: r#""#,
        location: r#"Lebanon"#,
        info: r#"Almond and semolina"#,
    },
    Cake {
        name: r#"Simnel cake"#,
        file: r#"Simnel cake 1.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Marzipan and dried fruit"#,
    },
    Cake {
        name: r#"Smoked salmon cheesecake"#,
        file: r#"Smoked salmon cheesecake.jpg"#,
        location: r#"United Kingdom (Scotland)"#,
        info: r#"Savory cheesecake containing smoked salmon"#,
    },
    Cake {
        name: r#"Smörgåstårta"#,
        file: r#"Smorgatarta.JPG"#,
        location: r#"Sweden, Estonia, Finland"#,
        info: r#"A cake that literally means "sandwich-cake" or "sandwich gateau" and it is a Scandinavian cuisine dish that is popular in Sweden, Estonia (as "võileivatort") and Finland (as "voileipäkakku"). This savoury cake is compositionally similar to a sandwich, but it has such a large amount of filling that it more resembles a layered cream cake with garnished top."#,
    },
    Cake {
        name: r#"Snowball cake"#,
        file: r#"Hostess-Sno-Ball-WS.jpg"#,
        location: r#"United States"#,
        info: r#"Marshmallow and coconut frosting"#,
    },
    Cake {
        name: r#"Snow skin mooncake"#,
        file: r#"SnowSkinMooncake1.JPG"#,
        location: r#"Hong Kong, China"#,
        info: r#"A Chinese food eaten during the Mid-Autumn Festival. It is a non-baked mooncake which originated in Hong Kong. The snow skin mooncake was developed by a bakery in Hong Kong, because the traditional mooncakes were made with salted duck egg yolks and lotus seed paste, resulting in very high sugar and oil content. It is also known as snowy mooncake, icy mooncake, and crystal mooncake."#,
    },
    Cake {
        name: r#"Soufflé"#,
        file: r#"Choco souffle.jpg"#,
        location: r#"France"#,
        info: r#"Cream sauce or purée with beaten egg whites"#,
    },
    Cake {
        name: r#"Spekkoek"#,
        file: r#"Spekkoek naturel en pandan.jpg"#,
        location: r#"Dutch East Indies"#,
        info: r#"Multi-layered, containing cinnamon, clove, mace and anise."#,
    },
    Cake {
        name: r#"Spice cake"#,
        file: r#"Spice Cake with sea foam frosting.jpg"#,
        location: r#"North America"#,
        info: r#"Spices such as cinnamon, cloves, allspice, ginger and/or mace"#,
    },
    Cake {
        name: r#"Spit cake"#,
        location: r#"unknown"#,
        file: r#"Kürtőskalács Budapest 2008.jpg"#,
        info: r#"A term for hollow, cylindrical cakes prepared in several countries"#,
    },
    Cake {
        name: r#"Sponge cake"#,
        file: r#"Sponge cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Flour, sugar and eggs."#,
    },
    Cake {
        name: r#"Saint Honoré cake"#,
        file: r#""#,
        location: r#"France"#,
        info: r#"Caramel and Chiboust cream"#,
    },
    Cake {
        name: r#"Stack cake"#,
        file: r#"Apple Stack Cake.jpg"#,
        location: r#"United States"#,
        info: r#"A cake that replaces a wedding cake."#,
    },
    Cake {
        name: r#"Streuselkuchen"#,
        file: r#"Streuselkuchen7.jpg"#,
        location: r#"Germany"#,
        info: r#"Streusel (butter, flour and sugar)"#,
    },
    Cake {
        name: r#"Studenterbrød"#,
        file: r#"Studenterbrød 2.jpg"#,
        location: r#"Denmark"#,
        info: r#"Trøffelmasse (crumbled cakes, cocoa-powder, sugar, butter, liquor), raspberry and chocolate."#,
    },
    Cake {
        name: r#"Sultana and cherry cake"#,
        file: r#""#,
        location: r#"United Kingdom"#,
        info: r#"Sultanas and glace cherries"#,
    },
    Cake {
        name: r#"Suncake (Taiwan)|Suncake"#,
        file: r#"Taichung Sun Cake.JPG"#,
        location: r#"Taiwan"#,
        info: r#"A popular Taiwanese dessert originally from the city of Taichung in Taiwan. The typical fillings consist of maltose (condensed malt sugar), and they are usually sold in special gift boxes as souvenirs for visitors."#,
    },
    Cake {
        name: r#"Swiss roll"#,
        file: r#"Swiss roll.jpg"#,
        location: r#"United Kingdom, not Switzerland as the name implies"#,
        info: r#"Jam and creamy filling; may come in different colors"#,
    },
    Cake {
        name: r#"Tarte Tatin"#,
        file: r#"Tarte tatin.jpg"#,
        location: r#"France"#,
        info: r#"Varies, commonly apple or pear"#,
    },
    Cake {
        name: r#"Tea loaf"#,
        file: r#""#,
        location: r#"United Kingdom"#,
        info: r#"Currants, sultanas and tea"#,
    },
    Cake {
        name: r#"Teacake"#,
        file: r#"Teacake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"Currants and sultana (grape)|sultanas"#,
    },
    Cake {
        name: r#"Tiramisu"#,
        file: r#"Tiramisu Fanes.jpg"#,
        location: r#"Italy"#,
        info: r#"Savoiardi and espresso"#,
    },
    Cake {
        name: r#"Tompouce"#,
        file: r#"Tom Pouce Dutch pastry.JPG"#,
        location: r#"Netherlands"#,
        info: r#"Cream, icing"#,
    },
    Cake {
        name: r#"Torta Tre Monti"#,
        file: r#""#,
        location: r#"Italy (San Marino)"#,
        info: r#"Hazelnuts"#,
    },
    Cake {
        name: r#"Tres leches cake"#,
        file: r#"Tres Leches.jpg"#,
        location: r#"Mexico, Costa Rica, Nicaragua"#,
        info: r#"Evaporated milk, condensed milk, and heavy cream or sour cream"#,
    },
    Cake {
        name: r#"Tunis cake"#,
        file: r#""#,
        location: r#"Scotland, Northern Ireland"#,
        info: r#"Chocolate and marzipan"#,
    },
    Cake {
        name: r#"Træstammer"#,
        file: r#"Træstammer 3.jpg"#,
        location: r#"Denmark"#,
        info: r#"Literally "wooden-logs". Trøffelmasse (crumbled cakes, cocoa-powder, sugar, butter, rum), marzipan and chocolate  Sweden has a similar cake known as Punsch-rolls."#,
    },
    Cake {
        name: r#"Upside-down cake"#,
        file: r#"Pineapple-upside-down-cake.jpg"#,
        location: r#"United Kingdom"#,
        info: r#"A cake that various ingredients like kiwi fruit or guava."#,
    },
    Cake {
        name: r#"Victoria sponge cake"#,
        file: r#"Cake_from_WHR(P).jpg"#,
        location: r#"United Kingdom"#,
        info: r#"A cake that was named after the Queen Victoria, who was known to enjoy a slice of the sponge cake with her afternoon tea. It is often referred to simply as "sponge cake", though it contains additional fat. A typical Victoria sponge consists of raspberry jam and whipped double cream or vanilla cream. The jam and cream are sandwiched between two sponge cakes; the top of the cake is not iced or decorated apart from a dusting of icing sugar. The Women's Institute publishes a variation on the Victoria sandwich that has raspberry jam as the filling and is dusted with caster sugar, not icing sugar."#,
    },
    Cake {
        name: r#"Wedding cake"#,
        file: r#"Whitweddingcake.jpg"#,
        location: r#"Unknown"#,
        info: r#"A traditional cake that is served at wedding receptions following dinner. In some parts of England, the wedding cake is served at a wedding breakfast, on the morning following the ceremony. In modern Western culture, the cake is usually on display and served to guests at the reception."#,
    },
    Cake {
        name: r#"Welsh cake"#,
        file: r#"Wesh cakes.jpg"#,
        location: r#"United Kingdom (Wales)"#,
        info: r#"Currants"#,
    },
    Cake {
        name: r#"Whoopie pies"#,
        file: r#"Whoopiepies1.jpg"#,
        location: r#"United States"#,
        info: r#"Cocoa, vanilla"#,
    },
];
