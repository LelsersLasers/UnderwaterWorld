// main hull
hull()
{
    // blok depan
    translate([25,10,10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([25,-10,10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([25,10,-10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([25,-10,-10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    // blok belakang
    translate([-60,10,10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([-60,-10,10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([-60,5,-10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    translate([-60,-5,-10]) rotate([0,90,0]) cylinder(h=20,d1=20,d2=20,$fa=1,center=true);
    // moncong
    translate([70,7,5]) rotate([0,20,-45]) scale([1,0.6,0.6]) sphere(d=30,center=true);
    translate([70,-7,5]) rotate([0,20,45]) scale([1,0.6,0.6]) sphere(d=30,center=true);
    translate([70,7,0]) rotate([0,-20,-45]) scale([1,0.6,0.6]) sphere(d=30,center=true);
    translate([70,-7,0]) rotate([0,-20,45]) scale([1,0.6,0.6]) sphere(d=30,center=true);
    // buntut
    translate([-120,0,10]) scale([1,0.5,0.5]) sphere(d=10,center=true);
}

// horizontal stab
hull()
{
    translate([-120,0,10]) scale([1,1,0.5]) rotate([90,0,0]) cylinder(h=40,d1=2,d2=2,$fa=1,center=true);
    translate([-110,0,10]) scale([1,1,0.5]) rotate([90,0,0]) cylinder(h=40,d1=5,d2=5,$fa=1,center=true);
}

// vertical stab
hull()
{
    translate([-120,0,10]) scale([1,0.5,1]) cylinder(h=40,d1=2,d2=2,$fa=1,center=true);
    translate([-110,0,10]) scale([1,0.5,1]) cylinder(h=40,d1=5,d2=5,$fa=1,center=true);
}

// bridge
hull()
{
    translate([0,0,25]) rotate([0,10,0]) scale([1,0.5,1]) cylinder(h=20,d1=20,d2=10,$fa=1,center=true);
    translate([20,0,25]) rotate([0,-20,0]) scale([1,0.5,1]) cylinder(h=20,d1=20,d2=10,$fa=1,center=true);
}

// fin
hull()
{
    translate([2,0,10]) scale([1,1,0.5]) rotate([90,0,0]) cylinder(h=80,d1=2,d2=2,$fa=1,center=true);
    translate([17,0,10]) scale([1,1,0.5]) rotate([90,0,0]) cylinder(h=80,d1=5,d2=5,$fa=1,center=true);
}

// ducted fan
difference()
{
    translate([-120,0,10]) rotate([0,90,0]) cylinder(h=20,d1=40,d2=40,$fa=1,center=true);
    translate([-120,0,10]) rotate([0,90,0]) cylinder(h=21,d1=38,d2=38,$fa=1,center=true);
}