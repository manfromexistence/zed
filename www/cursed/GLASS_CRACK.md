```html
<div class="wrapper">
  <img class="bg-image main animated" src="https://images.unsplash.com/photo-1442589031151-61d5645469d7?ixlib=rb-0.3.5&q=80&fm=jpg&crop=entropy&s=12154afe0240145e09bdad8a752d4fd6" alt="">
  
  <div class="drawing ui-corner-all animated">
       <canvas id="draw-refract"></canvas>
       <canvas id="draw-reflect"></canvas>
       <canvas id="draw-fractures"></canvas>
       <canvas id="draw-mainline"></canvas>
       <canvas id="draw-noise"></canvas>
       <canvas id="draw-debug" style="display: none;"></canvas>
       <div id="draw-picker"></div>
  </div>
  
  <div class="btn"><a href="#">Clear the canvas</a></div>
 
</div>
```

```js
(function($){

  function renderCrackEffectRefract(cvs, img, p1, p2, line)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           cp = line.cpt,

           ns = 3,
           td = 6,

           x1 = line.bbx1,
           y1 = line.bby1,
           w = line.bbwidth+ns*2,
           h = line.bbheight+ns*2;

       if (80 === 0) {
        return;
      }

       ctx.globalAlpha = 80 || 1;

       ctx.save();

       ctx.beginPath();
       ctx.moveTo(p1.x+ns*tx, p1.y+ns*ty);
       ctx.quadraticCurveTo(cp.x,cp.y,p2.x+ns*tx,p2.y+ns*ty);
       ctx.lineTo(p2.x-ns*tx, p2.y-ns*ty);
       ctx.quadraticCurveTo(cp.x,cp.y,p1.x-ns*tx,p1.y-ns*ty);
       ctx.closePath();
       ctx.clip();

       // Now copy a chunk of the image into the clipped region
       // Shift it slightly to create a minor refraction in the
       // image.

       // Eventually, the bounds will be right and it will stop
       // throwing errors ...

       try
       {
          if (x1+td*tx < 0) {x1 = -td*tx;}
          if (y1+td*ty < 0) {y1 = -td*ty;}
          if (w+x1+td*tx > ctx.canvas.window.innerWidth) {w = ctx.canvas.window.innerWidth - x1+td*tx;}
          if (h+y1+td*ty > ctx.canvas.window.innerHeight) {h = ctx.canvas.window.innerHeight - y1+td*ty;}

          ctx.drawImage(img, x1+td*tx, y1+td*ty, w, h, x1, y1, w, h);
       }
       catch (e)
       {
          // Bounds debugging
          // console.log('x1:'+x1+',mx:'+td*tx+',y1:'+y1+',my:'+td*ty+',w:'+w+',h:'+h);
       }

       ctx.restore();
    }

  function renderCrackEffectReflect(cvs, img, p1, p2, line, options)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           cp = line.cpt,

           dd = line.dl / 3,
           grd,
           clr = jQuery.Color('rgb(255,255,255)');

       if (0.3 === 0) return;

       ctx.globalAlpha = 0.3 || 1;

       try
       {
          grd = ctx.createLinearGradient(p1.x+dd*tx,p1.y+dd*ty,p1.x-dd*tx,p1.y-dd*ty);
       }
       catch (e)
       {
          // Bounds debugging
          console.log('x1:'+(p1.x+dd*tx)+',y1:'+(p1.y+dd*ty)+',x2:'+(p1.x-dd*tx)+',y2:'+(p1.y-dd*ty));
       }

       grd.addColorStop(0  , clr.alpha(0).toRgbaString());
       grd.addColorStop(0.5, clr.alpha(0.5).toRgbaString());
       grd.addColorStop(1  , clr.alpha(0).toRgbaString());

       ctx.fillStyle = grd;

       ctx.beginPath();
       ctx.moveTo(p1.x+dd*tx, p1.y+dd*ty);
       ctx.lineTo(p2.x+dd*tx, p2.y+dd*ty);
       ctx.lineTo(p2.x-dd*tx, p2.y-dd*ty);
       ctx.lineTo(p1.x-dd*tx, p1.y-dd*ty);
       ctx.closePath();
       ctx.fill();
    }

  function renderCrackEffectFractures(cvs, img, p1, p2, line, options)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           sx = line.sx,
           sy = line.sy,

           sz = 33,

           dl = line.dl,
           mp = dl / 2,

           mpp = line.mpp,
           cma = line.cma,
           mpl1 = line.mpl1,
           mpl2 = line.mpl2,

           s, p, c, w, h1, h2, t,

           clr = jQuery.Color('rgb(255,255,255');

       if (0.4 === 0) return;

       ctx.globalAlpha = 0.4 || 1;

       ctx.lineWidth = 1;

       for (s=0;s<dl;s++)
       {

          if (s < mpp*dl)
             c = cma * (1-Math.pow((mpl1-s)/mpl1, 2));
          else
             c = cma * (1-Math.pow((mpl2-(dl-s))/mpl2, 2));

          c /= 2;

          p = Math.pow((s > mp ? dl - s : s)/mp, 2);

          w = Math.random() * 1 + 1;
          h1 = sz - Math.random() * p * sz + 1;
          h2 = sz - Math.random() * p * sz + 1;
          t = Math.random() * 20 - 10;

          if (Math.random() > p-sz/mp)
          {
             ctx.fillStyle = clr.alpha(Math.round(Math.random() * 8 + 4) / 12).toRgbaString();

             ctx.beginPath();
             ctx.moveTo(p1.x + s*sx + c*tx,                 p1.y + s*sy + c*ty);
             ctx.lineTo(p1.x + (t+s+w/2)*sx + h1*tx + c*tx,   p1.y + (-t+s+w/2)*sy + h1*ty + c*ty);
             ctx.lineTo(p1.x + (s+w)*sx + c*tx,             p1.y + (s+w)*sy + c*ty);
             ctx.lineTo(p1.x + (-t+s+w/2)*sx - h2*tx + c*tx,   p1.y + (t+s+w/2)*sy - h2*ty + c*ty);
             ctx.closePath();
             ctx.fill();
          }

          s += mp * (p/2+0.5);
       }
    }

  function renderCrackEffectMainLine(cvs, img, p1, p2, line, options)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           cp = line.cpt,

           ns = 0.03 || 1,
           st = 0.14 || 1,
           hl = 0.2 || 0,
           tt = Math.random() * (ns*2) - (ns*2)/2,

           clr = jQuery.Color('rgb(255,255,255)'),
           nn = clr.lightness();

       if (65 === 0) return;

       ctx.globalAlpha = 65 || 1;

       ctx.lineWidth = 1;

       while (st > 0)
       {
          ctx.strokeStyle =
              clr
                .lightness( (nn > 0.5 ? nn : (1-nn)) * (1-hl*Math.random()) )
                .alpha(Math.round(Math.random() * 8 + 4) / 12)
                .toRgbaString();


          ctx.beginPath();
          ctx.moveTo(p1.x+(st+tt)*tx, p1.y+(st-tt)*ty);
          ctx.quadraticCurveTo(cp.x,cp.y,p2.x+(st-tt)*tx,p2.y+(st+tt)*ty);
          ctx.stroke();

          st--;
       }

    }

  function renderCrackEffectNoise(cvs, img, p1, p2, line, options)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           sx = line.sx,
           sy = line.sy,

           freq = 0.4,

           dl = line.dl,
           mp = dl / 2,

           mpp = line.mpp,
           cma = line.cma,
           mpl1 = line.mpl1,
           mpl2 = line.mpl2,

           dd = dl / 3,
           step = Math.ceil(dd * (1-(freq+0.5)/1.5) + 1),

           c, t, s, pos, cnt, m,

           clr = jQuery.Color('rgb(255,255,255)');

       if (1 === 0) return;

       ctx.globalAlpha = 1 || 1;

       ctx.lineWidth = 1;

       for (s=0;s<dl;s++)
       {
          if (s < mpp*dl)
             c = cma * (1-Math.pow((mpl1-s)/mpl1, 2));
          else
             c = cma * (1-Math.pow((mpl2-(dl-s))/mpl2, 2));

          c /= 2;

          for (t=-dd;t<dd;t++)
          {
             if (Math.random() > Math.abs(t) / dd )
             {
                cnt = Math.floor(Math.random()*4+0.5);
                m = Math.random() * 2 - 1;

                while (cnt >= 0)
                {
                   ctx.strokeStyle = clr.alpha(Math.round(Math.random() * 10 + 2) / 30).toRgbaString();

                   pos = Math.floor(Math.random()*5+0.5);

                   ctx.beginPath();
                   ctx.moveTo(p1.x + (s-pos)*sx + (m+t)*tx + c*tx, p1.y + (s-pos)*sy + (-m+t)*ty + c*ty);
                   ctx.lineTo(p1.x + (s+pos)*sx + (-m+t)*tx + c*tx, p1.y + (s+pos)*sy + (m+t)*ty + c*ty);
                   ctx.stroke();

                   cnt--;
                   pos++;
                }
             }

             t += Math.random()*step*2;
          }

          s += Math.random()*step*4;
       }
    }

  function renderCrackEffectDebug(cvs, img, p1, p2, line, options)
    {
       var ctx = cvs.getContext('2d'),

           tx = line.tx,
           ty = line.ty,

           sx = line.sx,
           sy = line.sy,

           mpp = line.mpp,
           cma = line.cma/2,
           mpl1 = line.mpl1,

           cp = line.cpt;

       ctx.strokeStyle = 'rgba(0,0,0,0)';
       ctx.fillStyle = 'rgba(0,0,0,0)';

       ctx.fillRect(p1.x, p1.y, 3, 3);

       ctx.beginPath();
       ctx.moveTo(p1.x, p1.y);
       ctx.lineTo(p2.x, p2.y);
       ctx.stroke();

       ctx.strokeStyle = 'rgba(0,0,0,0)';

       ctx.beginPath();
       ctx.moveTo(p1.x+mpl1*sx, p1.y+mpl1*sy);
       ctx.lineTo(p1.x+mpl1*sx+cma*tx, p1.y+mpl1*sy+cma*ty);
       ctx.stroke();

       ctx.strokeStyle = 'rgba(0,0,0,0)';

       ctx.beginPath();
       ctx.moveTo(p1.x+(mpl1-5)*sx+cma*tx, p1.y+(mpl1-5)*sy+cma*ty);
       ctx.lineTo(p1.x+(mpl1+5)*sx+cma*tx, p1.y+(mpl1+5)*sy+cma*ty);
       ctx.stroke();
    }

  function renderCrackEffectAll($canvas, $image, paths, options)
    {
       var i, line,
           len = paths.length;

       for (i=0;i<len;i++)
       {
          line = paths[i];


          renderCrackEffectRefract($canvas[0], $image[0], line.p1, line.p2, line.desc);

          renderCrackEffectReflect($canvas[1], $image[0], line.p1, line.p2, line.desc);

          renderCrackEffectFractures($canvas[2], $image[0], line.p1, line.p2, line.desc);

          renderCrackEffectMainLine($canvas[3], $image[0], line.p1, line.p2, line.desc);

          renderCrackEffectNoise($canvas[4], $image[0], line.p1, line.p2, line.desc);

          if ('Show') {renderCrackEffectDebug($canvas[5], $image[0], line.p1, line.p2, line.desc)};

       }

    }

  var _RAD = Math.PI / 180;

  function findPointOnCircle(c, r, a)
    {
       return {
           x: c.x + r * Math.cos(a*_RAD) - r * Math.sin(a*_RAD),
           y: c.y + r * Math.sin(a*_RAD) + r * Math.cos(a*_RAD)
         };
    }

  function describeLinePath(p1, p2, cv)
    {
       var o = {},
           ll,
           cv = 5*cv;

       o.dx = (p2.x - p1.x);
       o.dy = (p2.y - p1.y);
       o.dl = Math.sqrt(o.dx*o.dx + o.dy*o.dy);

       // Vectors
       o.sx = o.dx / o.dl;
       o.sy = o.dy / o.dl;

       o.tx = o.dy / o.dl;
       o.ty = -o.dx / o.dl;

       //Curvature
       o.mpp = Math.random() * 0.5 + 0.3;
       o.mpl1 = o.dl * o.mpp;
       o.mpl2 = o.dl - o.mpl1;

       ll = Math.log(o.dl*Math.E);
       o.cma = Math.random() * ll * cv - ll * cv / 2;
       o.cpt = {x: p1.x + o.sx*o.mpl1 + o.tx*o.cma, y: p1.y + o.sy*o.mpl1 + o.ty*o.cma};

       // Bounding box
       o.bbx1 = Math.min(p1.x, p2.x, o.cpt.x);
       o.bby1 = Math.min(p1.y, p2.y, o.cpt.y);
       o.bbx2 = Math.max(p1.x, p2.x, o.cpt.x);
       o.bby2 = Math.max(p1.y, p2.y, o.cpt.y);
       o.bbwidth = o.bbx2 - o.bbx1;
       o.bbheight = o.bby2 - o.bby1;

       return o;
    }

  function findCrackEffectPaths(options)
    {
       var imx = 0,
           imy = 0,
           imw = options.width,
           imh = options.height,
           ctx,
           main = [[]],
           lines = [],
           level = 1,
           maxl = 0,
           r = 15,
           c = options.center,
           pt1, pt2, ang, num, num2;

       /*
       * Part 1: Create a table of points that we can use to draw crack segments
       * between.  First, we need to find the number of lines that will run
       * outward from the center of the crack.  Each of these lines will be
       * staggered at various angles.  The points will be placed on these
       * lines at different intervals defined by the concentric circles
       * created by incrementing the starting radius.
       */

       num = 20;
       ang = 360/(num+1);

       while (main[0].length < num)
       {
          num2 = (ang*main[0].length)+10;
          pt2 = findPointOnCircle(c, 5, num2);
          main[0].push({angle: num2, point: pt2});
       }

       while(r < 500)
       {
          main[level]=[];
          for (num2=0;num2<num;num2++)
          {
             pt1 = main[level-1][num2];
             main[level][num2] = null;

             if (pt1)
             {
                if ((pt1.point.x > imx && pt1.point.x < imw) && (pt1.point.y > imy && pt1.point.y < imh))
                {
                   ang = pt1.angle + Math.random() * 10 / num - 10 / 2 / num;
                   if (ang > 350) ang = 350;

                   pt1 = pt1.point;

                   pt2 = findPointOnCircle(c, r+Math.random()*r/level-r/(level*2), ang);

                   //pt2.x = Math.max(imx, Math.min(imw, pt2.x));
                   //pt2.y = Math.max(imy, Math.min(imh, pt2.y));

                   main[level][num2] = {angle: ang, point: {x: pt2.x, y: pt2.y}};
                }
                else if (maxl == 0)
                {
                   maxl = level;
                }
             }
          }

          level++;
          r *= Math.random()*1.5 + (1.5 - 50 / 100);
       }

       /*
       * Part 2: Find the actual cracked lines between the points.
       * There are three lines that can be drawn:
       *
       *   a) The original lines from the center radiating out to the
       *      edges.  These are always drawn
       *   b) Lines connecting two adjacent points on the same circle
       *   c) Lines connecting two adjacent points on different circles
       *
       *   b & c are only drawn based a on random interval.  These
       *   lines create the web effect of the cracking.
       */

       if (maxl == 0) maxl = level;

       var l=1, g=0;

       for (;l<level;l++)
       {
          for (g=0;g<num;g++)
          {
             pt1 = main[l-1][g];
             pt2 = main[l][g];

             if (pt1 && pt2)
             {
                lines.push({
                     p1: {x:pt1.point.x, y:pt1.point.y},
                     p2: {x:pt2.point.x, y:pt2.point.y},
                     desc: describeLinePath(pt1.point, pt2.point, 30 / 100),
                     level: l
                   });

                if (Math.random() < (60 / 100))
                {
                   pt1 = main[l][(g+1)%num];
                   if (pt1)
                   {
                      lines.push({
                           p1: {x:pt2.point.x, y:pt2.point.y},
                           p2: {x:pt1.point.x, y:pt1.point.y},
                           desc: describeLinePath(pt2.point, pt1.point, 30 / 100),
                           level: l
                         });
                   }
                }

                if (l < level-1 &&
                    Math.random() < (30 / 100))
                {
                   pt1 = main[l+1][(g+1)%num];
                   if (pt1)
                   {
                      lines.push({
                           p1: {x:pt2.point.x, y:pt2.point.y},
                           p2: {x:pt1.point.x, y:pt1.point.y},
                           desc: describeLinePath(pt2.point, pt1.point, 30 / 100),
                           level: l
                         });
                   }
                }
             }
          }
       }

       return lines;
    }

  function clearDrawing($canvas)
    {
       $canvas.each(function()
         {
            var ctx = this.getContext('2d');
            ctx.clearRect(0,0,ctx.canvas.width,ctx.canvas.height);
         });
    }

  function validate()
    {
       var f = $('.c-field:not([fieldtype=none])'),
           b, g, pos,
           o = {};

       f.each(function ()
         {
            var $el = $(this), val=null;

            switch ($el.attr('fieldtype'))
            {
               case 'spinner' :
                 val = +$el.find('input')[0].value;
                 break;

               case 'slider' :
                 val = $el.slider('value');
                 break;
            }

            b = $el.attr('databind');
            if (b.indexOf('.') > 0)
            {
               g = b.split('.')[0];
               b = b.split('.')[1];

               o[g] = o[g] || {};
               o[g][b] = val;
            }
            else
            {
               o[b] = val;
            }
         });

       return o;
    }

  function fillImage(img, color, w, h)
    {
       var c = $('<canvas>')[0],
           ctx;

       c.width = w;
       c.height = h;

       ctx = c.getContext('2d');

       ctx.fillStyle = color;
       ctx.fillRect(0,0,w,h);

       img.src = c.toDataURL();
    }

  $(function()
    {
       $('.main')
          .one('load', init)
          .each(function() { /* Opera! */ if (this.complete) { $(this).trigger("load"); } });
    });

  /**********************************************************************************
  *   Init
  */

  function init()
    {
      var $canvas = $('canvas'),
          $image = $('.main'),
          options, paths, currentCenter;

      // Instead of 1418 originally used - $image.outerHeight(true)
      // Instead of 2556 originally used - $image.outerWidth(true)

      $canvas.each(function() {
        this.height = 600;
        this.width = 800;
      });

      $('#draw-picker').css({
        height: 600,
        width: 800
      });

      currentCenter = {
        x: 300,
        y: 300
      }

      $('#draw-picker').click(function(e) {
        var pos = $('.drawing').offset(),
            x = e.pageX - pos.left - 5,
            y = e.pageY - pos.top - 5;

        currentCenter = { x: x, y: y };
      });

      // Click on background image/video creates a new path
      $('#draw-picker').click(function() {
          if (options = validate())
          {
             options.height = 600;
             options.width = 800;
             options.center = currentCenter;
             options.debug = true;

             paths = findCrackEffectPaths(options);

             // clearDrawing($canvas);
             renderCrackEffectAll($canvas, $image, paths, options);
          }

        });
      console.log('init() run')
    }

  /**********************************************************************************
  *   Main
  */

  $(document).ready(function(){

    ///////////////////////
    // Change image on click

/*    $('#draw-picker').click(function(){

      var previous = $('.drawing').prev('.main'),
        $canvas = $('canvas');

      setTimeout(function(){
        $('.drawing').fadeOut('500');
        previous.fadeOut('500', function(){
          previous.prependTo($('.wrapper')).css('display','block');
          clearDrawing($canvas);
          $('.drawing').css('display','block');          
        });
      }, 800);

    });*/
    
    $('.btn').click(function(){
      
      var $canvas = $('canvas');      
      
      function clearDrawing($canvas) {
        $canvas.each(function() {
          var ctx = this.getContext('2d');
          ctx.clearRect(0,0,ctx.canvas.width,ctx.canvas.height);
        });
      }
      clearDrawing($canvas);
    
    });

  });

})(jQuery);
```

```css
@import "compass/css3";

/* ------------------------------------------- *\
	Main
\* ------------------------------------------- */
html {
	background: #DAD8DB;
  text-align: center;
}
body {
	font: 16px sans-serif;
	color: #000;
}
.wrapper {
  position: absolute;
  width: 800px;
  height: 600px;
  top: 100px;
  left: 50%;
  margin-left: -408px;
  border: 8px solid #000;
}
.btn {
  position: absolute;
  top: 615px;
  right: -8px;
  background: #000;
  border-bottom: 2px solid #fff;
  a {
    display: block;
    padding: 20px;
    text-decoration: none;
    color: #fff;
  }
}
/* ------------------------------------------- *\
	Background Image
\* ------------------------------------------- */
.bg-image {
	display: block;
	position: absolute;
  width: 800px;
  height: 600px;
}
/* ------------------------------------------- *\
	Drawing
\* ------------------------------------------- */
.drawing {
	position: relative;
}
.drawing > * {
	position: absolute;
  top: 0;
  left: 0;
}
```