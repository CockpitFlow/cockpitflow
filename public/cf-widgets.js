/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const t=globalThis,e=t.ShadowRoot&&(void 0===t.ShadyCSS||t.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,s=Symbol(),i=new WeakMap;let r=class{constructor(t,e,i){if(this._$cssResult$=!0,i!==s)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=t,this.t=e}get styleSheet(){let t=this.o;const s=this.t;if(e&&void 0===t){const e=void 0!==s&&1===s.length;e&&(t=i.get(s)),void 0===t&&((this.o=t=new CSSStyleSheet).replaceSync(this.cssText),e&&i.set(s,t))}return t}toString(){return this.cssText}};const o=(t,...e)=>{const i=1===t.length?t[0]:e.reduce((e,s,i)=>e+(t=>{if(!0===t._$cssResult$)return t.cssText;if("number"==typeof t)return t;throw Error("Value passed to 'css' function must be a 'css' function result: "+t+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(s)+t[i+1],t[0]);return new r(i,t,s)},n=e?t=>t:t=>t instanceof CSSStyleSheet?(t=>{let e="";for(const s of t.cssRules)e+=s.cssText;return(t=>new r("string"==typeof t?t:t+"",void 0,s))(e)})(t):t,{is:a,defineProperty:l,getOwnPropertyDescriptor:c,getOwnPropertyNames:h,getOwnPropertySymbols:d,getPrototypeOf:p}=Object,u=globalThis,f=u.trustedTypes,m=f?f.emptyScript:"",g=u.reactiveElementPolyfillSupport,_=(t,e)=>t,b={toAttribute(t,e){switch(e){case Boolean:t=t?m:null;break;case Object:case Array:t=null==t?t:JSON.stringify(t)}return t},fromAttribute(t,e){let s=t;switch(e){case Boolean:s=null!==t;break;case Number:s=null===t?null:Number(t);break;case Object:case Array:try{s=JSON.parse(t)}catch(t){s=null}}return s}},$=(t,e)=>!a(t,e),v={attribute:!0,type:String,converter:b,reflect:!1,useDefault:!1,hasChanged:$};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */Symbol.metadata??=Symbol("metadata"),u.litPropertyMetadata??=new WeakMap;let y=class extends HTMLElement{static addInitializer(t){this._$Ei(),(this.l??=[]).push(t)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(t,e=v){if(e.state&&(e.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(t)&&((e=Object.create(e)).wrapped=!0),this.elementProperties.set(t,e),!e.noAccessor){const s=Symbol(),i=this.getPropertyDescriptor(t,s,e);void 0!==i&&l(this.prototype,t,i)}}static getPropertyDescriptor(t,e,s){const{get:i,set:r}=c(this.prototype,t)??{get(){return this[e]},set(t){this[e]=t}};return{get:i,set(e){const o=i?.call(this);r?.call(this,e),this.requestUpdate(t,o,s)},configurable:!0,enumerable:!0}}static getPropertyOptions(t){return this.elementProperties.get(t)??v}static _$Ei(){if(this.hasOwnProperty(_("elementProperties")))return;const t=p(this);t.finalize(),void 0!==t.l&&(this.l=[...t.l]),this.elementProperties=new Map(t.elementProperties)}static finalize(){if(this.hasOwnProperty(_("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(_("properties"))){const t=this.properties,e=[...h(t),...d(t)];for(const s of e)this.createProperty(s,t[s])}const t=this[Symbol.metadata];if(null!==t){const e=litPropertyMetadata.get(t);if(void 0!==e)for(const[t,s]of e)this.elementProperties.set(t,s)}this._$Eh=new Map;for(const[t,e]of this.elementProperties){const s=this._$Eu(t,e);void 0!==s&&this._$Eh.set(s,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(t){const e=[];if(Array.isArray(t)){const s=new Set(t.flat(1/0).reverse());for(const t of s)e.unshift(n(t))}else void 0!==t&&e.push(n(t));return e}static _$Eu(t,e){const s=e.attribute;return!1===s?void 0:"string"==typeof s?s:"string"==typeof t?t.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(t=>this.enableUpdating=t),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(t=>t(this))}addController(t){(this._$EO??=new Set).add(t),void 0!==this.renderRoot&&this.isConnected&&t.hostConnected?.()}removeController(t){this._$EO?.delete(t)}_$E_(){const t=new Map,e=this.constructor.elementProperties;for(const s of e.keys())this.hasOwnProperty(s)&&(t.set(s,this[s]),delete this[s]);t.size>0&&(this._$Ep=t)}createRenderRoot(){const s=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return((s,i)=>{if(e)s.adoptedStyleSheets=i.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const e of i){const i=document.createElement("style"),r=t.litNonce;void 0!==r&&i.setAttribute("nonce",r),i.textContent=e.cssText,s.appendChild(i)}})(s,this.constructor.elementStyles),s}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(t=>t.hostConnected?.())}enableUpdating(t){}disconnectedCallback(){this._$EO?.forEach(t=>t.hostDisconnected?.())}attributeChangedCallback(t,e,s){this._$AK(t,s)}_$ET(t,e){const s=this.constructor.elementProperties.get(t),i=this.constructor._$Eu(t,s);if(void 0!==i&&!0===s.reflect){const r=(void 0!==s.converter?.toAttribute?s.converter:b).toAttribute(e,s.type);this._$Em=t,null==r?this.removeAttribute(i):this.setAttribute(i,r),this._$Em=null}}_$AK(t,e){const s=this.constructor,i=s._$Eh.get(t);if(void 0!==i&&this._$Em!==i){const t=s.getPropertyOptions(i),r="function"==typeof t.converter?{fromAttribute:t.converter}:void 0!==t.converter?.fromAttribute?t.converter:b;this._$Em=i;const o=r.fromAttribute(e,t.type);this[i]=o??this._$Ej?.get(i)??o,this._$Em=null}}requestUpdate(t,e,s,i=!1,r){if(void 0!==t){const o=this.constructor;if(!1===i&&(r=this[t]),s??=o.getPropertyOptions(t),!((s.hasChanged??$)(r,e)||s.useDefault&&s.reflect&&r===this._$Ej?.get(t)&&!this.hasAttribute(o._$Eu(t,s))))return;this.C(t,e,s)}!1===this.isUpdatePending&&(this._$ES=this._$EP())}C(t,e,{useDefault:s,reflect:i,wrapped:r},o){s&&!(this._$Ej??=new Map).has(t)&&(this._$Ej.set(t,o??e??this[t]),!0!==r||void 0!==o)||(this._$AL.has(t)||(this.hasUpdated||s||(e=void 0),this._$AL.set(t,e)),!0===i&&this._$Em!==t&&(this._$Eq??=new Set).add(t))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const t=this.scheduleUpdate();return null!=t&&await t,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[t,e]of this._$Ep)this[t]=e;this._$Ep=void 0}const t=this.constructor.elementProperties;if(t.size>0)for(const[e,s]of t){const{wrapped:t}=s,i=this[e];!0!==t||this._$AL.has(e)||void 0===i||this.C(e,void 0,s,i)}}let t=!1;const e=this._$AL;try{t=this.shouldUpdate(e),t?(this.willUpdate(e),this._$EO?.forEach(t=>t.hostUpdate?.()),this.update(e)):this._$EM()}catch(e){throw t=!1,this._$EM(),e}t&&this._$AE(e)}willUpdate(t){}_$AE(t){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(t)),this.updated(t)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(t){return!0}update(t){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(t){}firstUpdated(t){}};y.elementStyles=[],y.shadowRootOptions={mode:"open"},y[_("elementProperties")]=new Map,y[_("finalized")]=new Map,g?.({ReactiveElement:y}),(u.reactiveElementVersions??=[]).push("2.1.2");
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const x=globalThis,A=t=>t,w=x.trustedTypes,S=w?w.createPolicy("lit-html",{createHTML:t=>t}):void 0,E="$lit$",k=`lit$${Math.random().toFixed(9).slice(2)}$`,C="?"+k,M=`<${C}>`,O=document,P=()=>O.createComment(""),U=t=>null===t||"object"!=typeof t&&"function"!=typeof t,N=Array.isArray,T="[ \t\n\f\r]",z=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,R=/-->/g,H=/>/g,B=RegExp(`>|${T}(?:([^\\s"'>=/]+)(${T}*=${T}*(?:[^ \t\n\f\r"'\`<>=]|("|')|))|$)`,"g"),L=/'/g,D=/"/g,I=/^(?:script|style|textarea|title)$/i,j=(t=>(e,...s)=>({_$litType$:t,strings:e,values:s}))(1),V=Symbol.for("lit-noChange"),q=Symbol.for("lit-nothing"),W=new WeakMap,J=O.createTreeWalker(O,129);function F(t,e){if(!N(t)||!t.hasOwnProperty("raw"))throw Error("invalid template strings array");return void 0!==S?S.createHTML(e):e}const K=(t,e)=>{const s=t.length-1,i=[];let r,o=2===e?"<svg>":3===e?"<math>":"",n=z;for(let e=0;e<s;e++){const s=t[e];let a,l,c=-1,h=0;for(;h<s.length&&(n.lastIndex=h,l=n.exec(s),null!==l);)h=n.lastIndex,n===z?"!--"===l[1]?n=R:void 0!==l[1]?n=H:void 0!==l[2]?(I.test(l[2])&&(r=RegExp("</"+l[2],"g")),n=B):void 0!==l[3]&&(n=B):n===B?">"===l[0]?(n=r??z,c=-1):void 0===l[1]?c=-2:(c=n.lastIndex-l[2].length,a=l[1],n=void 0===l[3]?B:'"'===l[3]?D:L):n===D||n===L?n=B:n===R||n===H?n=z:(n=B,r=void 0);const d=n===B&&t[e+1].startsWith("/>")?" ":"";o+=n===z?s+M:c>=0?(i.push(a),s.slice(0,c)+E+s.slice(c)+k+d):s+k+(-2===c?e:d)}return[F(t,o+(t[s]||"<?>")+(2===e?"</svg>":3===e?"</math>":"")),i]};class X{constructor({strings:t,_$litType$:e},s){let i;this.parts=[];let r=0,o=0;const n=t.length-1,a=this.parts,[l,c]=K(t,e);if(this.el=X.createElement(l,s),J.currentNode=this.el.content,2===e||3===e){const t=this.el.content.firstChild;t.replaceWith(...t.childNodes)}for(;null!==(i=J.nextNode())&&a.length<n;){if(1===i.nodeType){if(i.hasAttributes())for(const t of i.getAttributeNames())if(t.endsWith(E)){const e=c[o++],s=i.getAttribute(t).split(k),n=/([.?@])?(.*)/.exec(e);a.push({type:1,index:r,name:n[2],strings:s,ctor:"."===n[1]?tt:"?"===n[1]?et:"@"===n[1]?st:Q}),i.removeAttribute(t)}else t.startsWith(k)&&(a.push({type:6,index:r}),i.removeAttribute(t));if(I.test(i.tagName)){const t=i.textContent.split(k),e=t.length-1;if(e>0){i.textContent=w?w.emptyScript:"";for(let s=0;s<e;s++)i.append(t[s],P()),J.nextNode(),a.push({type:2,index:++r});i.append(t[e],P())}}}else if(8===i.nodeType)if(i.data===C)a.push({type:2,index:r});else{let t=-1;for(;-1!==(t=i.data.indexOf(k,t+1));)a.push({type:7,index:r}),t+=k.length-1}r++}}static createElement(t,e){const s=O.createElement("template");return s.innerHTML=t,s}}function G(t,e,s=t,i){if(e===V)return e;let r=void 0!==i?s._$Co?.[i]:s._$Cl;const o=U(e)?void 0:e._$litDirective$;return r?.constructor!==o&&(r?._$AO?.(!1),void 0===o?r=void 0:(r=new o(t),r._$AT(t,s,i)),void 0!==i?(s._$Co??=[])[i]=r:s._$Cl=r),void 0!==r&&(e=G(t,r._$AS(t,e.values),r,i)),e}class Y{constructor(t,e){this._$AV=[],this._$AN=void 0,this._$AD=t,this._$AM=e}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(t){const{el:{content:e},parts:s}=this._$AD,i=(t?.creationScope??O).importNode(e,!0);J.currentNode=i;let r=J.nextNode(),o=0,n=0,a=s[0];for(;void 0!==a;){if(o===a.index){let e;2===a.type?e=new Z(r,r.nextSibling,this,t):1===a.type?e=new a.ctor(r,a.name,a.strings,this,t):6===a.type&&(e=new it(r,this,t)),this._$AV.push(e),a=s[++n]}o!==a?.index&&(r=J.nextNode(),o++)}return J.currentNode=O,i}p(t){let e=0;for(const s of this._$AV)void 0!==s&&(void 0!==s.strings?(s._$AI(t,s,e),e+=s.strings.length-2):s._$AI(t[e])),e++}}class Z{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(t,e,s,i){this.type=2,this._$AH=q,this._$AN=void 0,this._$AA=t,this._$AB=e,this._$AM=s,this.options=i,this._$Cv=i?.isConnected??!0}get parentNode(){let t=this._$AA.parentNode;const e=this._$AM;return void 0!==e&&11===t?.nodeType&&(t=e.parentNode),t}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(t,e=this){t=G(this,t,e),U(t)?t===q||null==t||""===t?(this._$AH!==q&&this._$AR(),this._$AH=q):t!==this._$AH&&t!==V&&this._(t):void 0!==t._$litType$?this.$(t):void 0!==t.nodeType?this.T(t):(t=>N(t)||"function"==typeof t?.[Symbol.iterator])(t)?this.k(t):this._(t)}O(t){return this._$AA.parentNode.insertBefore(t,this._$AB)}T(t){this._$AH!==t&&(this._$AR(),this._$AH=this.O(t))}_(t){this._$AH!==q&&U(this._$AH)?this._$AA.nextSibling.data=t:this.T(O.createTextNode(t)),this._$AH=t}$(t){const{values:e,_$litType$:s}=t,i="number"==typeof s?this._$AC(t):(void 0===s.el&&(s.el=X.createElement(F(s.h,s.h[0]),this.options)),s);if(this._$AH?._$AD===i)this._$AH.p(e);else{const t=new Y(i,this),s=t.u(this.options);t.p(e),this.T(s),this._$AH=t}}_$AC(t){let e=W.get(t.strings);return void 0===e&&W.set(t.strings,e=new X(t)),e}k(t){N(this._$AH)||(this._$AH=[],this._$AR());const e=this._$AH;let s,i=0;for(const r of t)i===e.length?e.push(s=new Z(this.O(P()),this.O(P()),this,this.options)):s=e[i],s._$AI(r),i++;i<e.length&&(this._$AR(s&&s._$AB.nextSibling,i),e.length=i)}_$AR(t=this._$AA.nextSibling,e){for(this._$AP?.(!1,!0,e);t!==this._$AB;){const e=A(t).nextSibling;A(t).remove(),t=e}}setConnected(t){void 0===this._$AM&&(this._$Cv=t,this._$AP?.(t))}}class Q{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(t,e,s,i,r){this.type=1,this._$AH=q,this._$AN=void 0,this.element=t,this.name=e,this._$AM=i,this.options=r,s.length>2||""!==s[0]||""!==s[1]?(this._$AH=Array(s.length-1).fill(new String),this.strings=s):this._$AH=q}_$AI(t,e=this,s,i){const r=this.strings;let o=!1;if(void 0===r)t=G(this,t,e,0),o=!U(t)||t!==this._$AH&&t!==V,o&&(this._$AH=t);else{const i=t;let n,a;for(t=r[0],n=0;n<r.length-1;n++)a=G(this,i[s+n],e,n),a===V&&(a=this._$AH[n]),o||=!U(a)||a!==this._$AH[n],a===q?t=q:t!==q&&(t+=(a??"")+r[n+1]),this._$AH[n]=a}o&&!i&&this.j(t)}j(t){t===q?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,t??"")}}class tt extends Q{constructor(){super(...arguments),this.type=3}j(t){this.element[this.name]=t===q?void 0:t}}class et extends Q{constructor(){super(...arguments),this.type=4}j(t){this.element.toggleAttribute(this.name,!!t&&t!==q)}}class st extends Q{constructor(t,e,s,i,r){super(t,e,s,i,r),this.type=5}_$AI(t,e=this){if((t=G(this,t,e,0)??q)===V)return;const s=this._$AH,i=t===q&&s!==q||t.capture!==s.capture||t.once!==s.once||t.passive!==s.passive,r=t!==q&&(s===q||i);i&&this.element.removeEventListener(this.name,this,s),r&&this.element.addEventListener(this.name,this,t),this._$AH=t}handleEvent(t){"function"==typeof this._$AH?this._$AH.call(this.options?.host??this.element,t):this._$AH.handleEvent(t)}}class it{constructor(t,e,s){this.element=t,this.type=6,this._$AN=void 0,this._$AM=e,this.options=s}get _$AU(){return this._$AM._$AU}_$AI(t){G(this,t)}}const rt=x.litHtmlPolyfillSupport;rt?.(X,Z),(x.litHtmlVersions??=[]).push("3.3.2");const ot=globalThis;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */class nt extends y{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const t=super.createRenderRoot();return this.renderOptions.renderBefore??=t.firstChild,t}update(t){const e=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(t),this._$Do=((t,e,s)=>{const i=s?.renderBefore??e;let r=i._$litPart$;if(void 0===r){const t=s?.renderBefore??null;i._$litPart$=r=new Z(e.insertBefore(P(),t),t,void 0,s??{})}return r._$AI(t),r})(e,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return V}}nt._$litElement$=!0,nt.finalized=!0,ot.litElementHydrateSupport?.({LitElement:nt});const at=ot.litElementPolyfillSupport;at?.({LitElement:nt}),(ot.litElementVersions??=[]).push("4.2.2");class lt extends nt{static properties={cmd:{type:String},dataref:{type:String},value:{type:Number},min:{type:Number},max:{type:Number},label:{type:String},unit:{type:String},color:{type:String},disabled:{type:Boolean}};static styles=o`
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      font-family: 'Segoe UI', system-ui, sans-serif;
      color: #c8ccd4;
      overflow: hidden;
      --cf-accent: #4a9eff;
      --cf-green: #44dd66;
      --cf-amber: #ddaa44;
      --cf-red: #ee4444;
      --cf-dim: #5a5e66;
      --cf-bg: #111317;
      --cf-surface: #1c1e22;
      --cf-border: #2a2d33;
    }
    :host([disabled]) { opacity: .3; pointer-events: none; }
    .label {
      font-family: 'Orbitron', sans-serif;
      font-size: 7px; font-weight: 700;
      color: var(--cf-dim); letter-spacing: 1px;
    }
    .value {
      font-family: 'Share Tech Mono', monospace;
      font-size: 10px; font-weight: 700;
      color: var(--cf-accent);
    }
  `;constructor(){super(),this.cmd="",this.dataref="",this.value=0,this.min=0,this.max=1,this.label="",this.unit="",this.color="accent",this.disabled=!1}_c(t){const e={accent:"--cf-accent",red:"--cf-red",amber:"--cf-amber",green:"--cf-green"};return`var(${e[t]||e.accent})`}_send(t){this.value=t,this.dispatchEvent(new CustomEvent("cf-input",{detail:{cmd:this.cmd,value:t},bubbles:!0,composed:!0}))}setValue(t){this.value=t}}class ct extends lt{static styles=[lt.styles,o`
    :host { gap: 2px; cursor: grab; touch-action: none; }
    .track {
      flex: 1; width: 100%; position: relative;
    }
    .rail {
      position: absolute; left: 50%; transform: translateX(-50%);
      width: 10px; height: 100%;
      background: linear-gradient(180deg, #0a0b0d, #161820, #0a0b0d);
      border: 1px solid #1e2028; border-radius: 5px;
    }
    .fill {
      position: absolute; left: 50%; transform: translateX(-50%);
      width: 6px; bottom: 0; border-radius: 3px; opacity: .5;
    }
    .thumb {
      position: absolute; left: 50%; transform: translate(-50%, 50%);
      width: 30px; height: 14px; border-radius: 3px;
      border: 1px solid; z-index: 2; pointer-events: none;
      box-shadow: 0 2px 6px rgba(0,0,0,.6), inset 0 1px 0 rgba(255,255,255,.08);
    }
    .ticks {
      position: absolute; left: 50%; transform: translateX(-50%);
      width: 20px; height: 100%; display: flex; flex-direction: column;
      justify-content: space-between; padding: 4px 0; pointer-events: none;
    }
    .tick { width: 100%; height: 1px; background: rgba(255,255,255,.06); }
  `];_dragging=!1;render(){const t=(this.value-this.min)/(this.max-this.min)*100,e=this._c(this.color);return j`
      ${this.label?j`<span class="label">${this.label}</span>`:""}
      <div class="track" @pointerdown=${this._down} @pointermove=${this._move}
           @pointerup=${this._up} @pointercancel=${this._up}>
        <div class="rail"></div>
        <div class="fill" style="height:${t}%;background:${e}"></div>
        <div class="ticks">${[0,1,2,3,4].map(()=>j`<div class="tick"></div>`)}</div>
        <div class="thumb" style="bottom:${t}%;background:linear-gradient(180deg,${"red"===this.color?"#cc3030,#771818":"amber"===this.color?"#ccaa22,#776610":"#3a3d44,#22242a"});border-color:${e}"></div>
      </div>
      <span class="value">${Math.round(t)}%</span>
    `}_down(t){t.preventDefault(),this._dragging=!0,t.target.closest(".track")?.setPointerCapture(t.pointerId),this._calc(t)}_move(t){this._dragging&&this._calc(t)}_up(){this._dragging=!1}_calc(t){const e=this.renderRoot.querySelector(".track").getBoundingClientRect(),s=1-(t.clientY-e.top)/e.height,i=this.min+Math.max(0,Math.min(1,s))*(this.max-this.min);this._send(i)}}customElements.define("cf-slider",ct);class ht extends lt{static styles=[lt.styles,o`
    :host { gap: 3px; cursor: pointer; }
    .pill {
      width: 26px; height: 13px; border-radius: 7px;
      background: var(--cf-surface); border: 1px solid var(--cf-border);
      position: relative; transition: all .12s;
    }
    .pill::after {
      content: ''; position: absolute; width: 9px; height: 9px; border-radius: 50%;
      top: 1px; left: 1px; background: var(--cf-dim); transition: all .12s;
    }
    :host([on]) .pill { border-color: var(--color); background: color-mix(in srgb, var(--color) 8%, transparent); }
    :host([on]) .pill::after { left: 14px; background: var(--color); box-shadow: 0 0 4px var(--color); }
  `];static properties={...lt.properties,on:{type:Boolean,reflect:!0}};constructor(){super(),this.on=!1}render(){return j`
      ${this.label?j`<span class="label">${this.label}</span>`:""}
      <div class="pill" style="--color:${this._c(this.color)}" @click=${this._tap}></div>
    `}_tap(){this.on=!this.on,this._send(this.on?1:0)}setValue(t){this.on=t>.5,this.value=t}}customElements.define("cf-toggle",ht);class dt extends lt{static properties={...lt.properties,format:{type:String},thresholds:{type:String}};static styles=[lt.styles,o`
    :host { flex-direction: row; gap: 4px; padding: 2px 8px; }
    .lbl { font-family: 'Orbitron', sans-serif; font-size: 7px; font-weight: 500; color: var(--cf-dim); letter-spacing: 1px; width: 26px; }
    .val {
      font-family: 'Share Tech Mono', monospace; font-size: 19px; font-weight: 700;
      color: var(--cf-green); text-align: right; flex: 1;
      text-shadow: 0 0 8px rgba(68,221,102,.1);
    }
    .val.warn { color: var(--cf-amber); }
    .val.danger { color: var(--cf-red); }
    .unit { font-size: 7px; color: var(--cf-dim); width: 22px; margin-left: 3px; }
  `];constructor(){super(),this.format="int",this.thresholds="[]"}render(){const t=this._format(this.value),e=this._threshold(this.value);return j`
      ${this.label?j`<span class="lbl">${this.label}</span>`:""}
      <span class="val ${e}">${t}</span>
      ${this.unit?j`<span class="unit">${this.unit}</span>`:""}
    `}_format(t){if(null==t)return"---";switch(this.format){case"heading":return String(Math.round(t)).padStart(3,"0");case"signed":{const e=Math.round(t);return(e>=0?"+":"")+e}case"decimal":return t.toFixed(1);default:return Math.round(t)}}_threshold(t){try{const e=JSON.parse(this.thresholds);for(const s of e)if(t<s.below)return"red"===s.color?"danger":"warn"}catch{}return""}}customElements.define("cf-display",dt);class pt extends lt{static properties={...lt.properties,momentary:{type:Boolean}};static styles=[lt.styles,o`
    :host { cursor: pointer; }
    .btn {
      padding: 8px 12px; background: var(--cf-surface);
      border: 1px solid var(--cf-border); border-radius: 4px;
      color: var(--cf-dim); font-family: 'Share Tech Mono', monospace;
      font-size: 11px; font-weight: 700; letter-spacing: .5px;
      text-align: center; transition: all .12s; width: 100%;
    }
    .btn:active, :host([pressed]) .btn {
      border-color: var(--color); color: var(--color);
      background: color-mix(in srgb, var(--color) 6%, transparent);
    }
  `];static properties={...lt.properties,momentary:{type:Boolean},pressed:{type:Boolean,reflect:!0}};constructor(){super(),this.momentary=!1,this.pressed=!1}render(){return j`
      <div class="btn" style="--color:${this._c(this.color)}"
        @pointerdown=${this._down} @pointerup=${this._up} @pointerleave=${this._up}>
        ${this.label||this.cmd}
      </div>
    `}_down(t){t.preventDefault(),this.pressed=!0,this._send(1)}_up(){this.momentary&&this.pressed?(this.pressed=!1,this._send(0)):this.momentary||(this.pressed=!1)}}customElements.define("cf-button",pt);class ut extends lt{static properties={...lt.properties,options:{type:String},spring:{type:String}};static styles=[lt.styles,o`
    :host { gap: 3px; }
    .group { display: flex; gap: 2px; flex-wrap: wrap; justify-content: center; }
    .opt {
      padding: 4px 8px; background: var(--cf-surface);
      border: 1px solid var(--cf-border); border-radius: 3px;
      color: var(--cf-dim); font-family: 'Share Tech Mono', monospace;
      font-size: 9px; font-weight: 700; cursor: pointer; transition: all .12s;
    }
    .opt.on {
      border-color: var(--cf-accent); color: var(--cf-accent);
      background: rgba(74,158,255,.06);
    }
    .opt.spring { border-color: #422; color: #966; }
    .opt.spring:active { border-color: var(--cf-red); color: var(--cf-red); }
  `];constructor(){super(),this.options="[]",this.spring=""}render(){let t=[];try{t=JSON.parse(this.options)}catch{}const e=this.spring?JSON.parse(this.spring):null;return j`
      ${this.label?j`<span class="label">${this.label}</span>`:""}
      <div class="group">
        ${t.map(t=>{const s=e&&t.value===e.from,i=this.value===t.value;return j`<button class="opt ${i?"on":""} ${s?"spring":""}"
            @pointerdown=${()=>this._select(t.value)}
            @pointerup=${()=>s&&this._springBack(e)}
            @pointerleave=${()=>s&&this.value===t.value&&this._springBack(e)}
          >${t.label}</button>`})}
      </div>
    `}_select(t){this._send(t)}_springBack(t){this._send(t.to)}}customElements.define("cf-button-group",ut);class ft extends lt{static properties={...lt.properties,step:{type:Number},format:{type:String}};static styles=[lt.styles,o`
    :host { gap: 3px; }
    .knob-row { display: flex; align-items: center; gap: 4px; }
    .knob-btn {
      width: 28px; height: 28px; border-radius: 50%;
      background: var(--cf-surface); border: 1px solid var(--cf-border);
      color: var(--cf-dim); font-size: 14px; cursor: pointer;
      display: flex; align-items: center; justify-content: center;
      transition: all .12s;
    }
    .knob-btn:active { border-color: var(--cf-accent); color: var(--cf-accent); }
    .knob-val {
      font-family: 'Share Tech Mono', monospace; font-size: 15px;
      font-weight: 700; color: var(--cf-accent); min-width: 40px; text-align: center;
    }
  `];_timer=null;constructor(){super(),this.step=1,this.format="int"}render(){return j`
      ${this.label?j`<span class="label">${this.label}</span>`:""}
      <div class="knob-row">
        <button class="knob-btn" @pointerdown=${()=>this._start(-1)} @pointerup=${this._stop} @pointerleave=${this._stop}>&#8722;</button>
        <span class="knob-val">${this._fmt(this.value)}</span>
        <button class="knob-btn" @pointerdown=${()=>this._start(1)} @pointerup=${this._stop} @pointerleave=${this._stop}>+</button>
      </div>
    `}_fmt(t){return"heading"===this.format?String(Math.round((t%360+360)%360)).padStart(3,"0"):"decimal"===this.format?t.toFixed(1):Math.round(t)}_nudge(t){let e=this.value+t*this.step;e<this.min&&(e=this.min),e>this.max&&(e=this.max),this._send(e)}_start(t){this._nudge(t),this._timer=setInterval(()=>this._nudge(t),120)}_stop=()=>{clearInterval(this._timer)}}customElements.define("cf-knob",ft);class mt extends lt{static properties={...lt.properties,"cmd-ail":{type:String},"cmd-elv":{type:String},sensitivity:{type:Number},deadzone:{type:Number},modes:{type:String}};static styles=[lt.styles,o`
    :host { gap: 4px; }
    .bar { display: flex; gap: 5px; align-items: center; }
    .yk-btn {
      padding: 3px 10px; background: var(--cf-bg); border: 1px solid var(--cf-border);
      border-radius: 3px; font-family: 'Orbitron', sans-serif; font-size: 7px; font-weight: 500;
      color: var(--cf-dim); letter-spacing: 1px; cursor: pointer; transition: all .12s;
    }
    .yk-btn.on { border-color: var(--cf-accent); color: var(--cf-accent); }
    .yk-btn.lk { border-color: var(--cf-red); color: var(--cf-red); }
    .pad {
      width: min(42vw, calc(100dvh - 110px)); height: min(42vw, calc(100dvh - 110px));
      max-width: 260px; max-height: 260px; border-radius: 50%;
      background: radial-gradient(circle, #181a1f, #101216 60%, #0c0e12);
      border: 1.5px solid var(--cf-border);
      box-shadow: inset 0 0 40px rgba(0,0,0,.4); position: relative; touch-action: none;
    }
    .pad::before {
      content: ''; position: absolute; inset: 0; border-radius: 50%; pointer-events: none;
      background:
        linear-gradient(90deg, transparent 49.6%, rgba(255,255,255,.03) 49.6%, rgba(255,255,255,.03) 50.4%, transparent 50.4%),
        linear-gradient(0deg, transparent 49.6%, rgba(255,255,255,.03) 49.6%, rgba(255,255,255,.03) 50.4%, transparent 50.4%);
    }
    .dot {
      position: absolute; width: 18px; height: 18px; border-radius: 50%;
      background: radial-gradient(circle at 38% 32%, #6abfff, #2a8aee);
      box-shadow: 0 0 10px rgba(74,170,255,.3);
      transform: translate(-50%, -50%); top: 50%; left: 50%;
      pointer-events: none; z-index: 2;
    }
    .dot.smooth { transition: top .08s, left .08s; }
    .rd { display: flex; gap: 14px; }
    .rv { font-family: 'Share Tech Mono', monospace; font-size: 11px; font-weight: 700; color: var(--cf-accent); }
    .rv small { color: var(--cf-dim); font-size: 7px; margin-right: 3px; }
  `];_touching=!1;_ail=0;_elv=0;_mode="touch";_locked=!1;_gyroOk=!1;_gbc=0;_ggc=0;_sendTimer=null;constructor(){super(),this["cmd-ail"]="AILERON",this["cmd-elv"]="ELEVATOR",this.sensitivity=1,this.deadzone=.05,this.modes='["touch","gyro"]'}render(){let t=[];try{t=JSON.parse(this.modes)}catch{}return j`
      <div class="bar">
        ${t.map((t,e)=>j`<button class="yk-btn ${this._mode===t?"on":""}" @click=${()=>this._setMode(t)}>${t.toUpperCase().slice(0,3)}</button>`)}
        <button class="yk-btn" @click=${this._center}>CTR</button>
        <button class="yk-btn ${this._locked?"lk":""}" @click=${this._toggleLock}>${this._locked?"UNLK":"LOCK"}</button>
      </div>
      <div class="pad" @pointerdown=${this._down} @pointermove=${this._move} @pointerup=${this._up} @pointercancel=${this._up}>
        <div class="dot ${"gyro"===this._mode?"smooth":""}" style="left:${50+44*this._ail}%;top:${50+44*this._elv}%"></div>
      </div>
      <div class="rd">
        <span class="rv"><small>AIL</small>${(this._ail>=0?"+":"")+this._ail.toFixed(2)}</span>
        <span class="rv"><small>ELV</small>${(this._elv>=0?"+":"")+this._elv.toFixed(2)}</span>
      </div>
    `}_dz(t){const e=this.deadzone;return Math.abs(t)<e?0:(t>0?1:-1)*(Math.abs(t)-e)/(1-e)}_set(t,e){if(this._locked)return;const s=this.sensitivity;this._ail=Math.max(-1,Math.min(1,this._dz(t*s))),this._elv=Math.max(-1,Math.min(1,this._dz(e*s))),this.requestUpdate(),this._sendTimer||(this._sendTimer=setTimeout(()=>{this._sendTimer=null,this.dispatchEvent(new CustomEvent("cf-input",{detail:{cmd:this["cmd-ail"],value:this._ail},bubbles:!0,composed:!0})),this.dispatchEvent(new CustomEvent("cf-input",{detail:{cmd:this["cmd-elv"],value:this._elv},bubbles:!0,composed:!0}))},33))}_reset(){this._set(0,0)}_down(t){"touch"!==this._mode||this._locked||(t.preventDefault(),this._touching=!0,t.target.closest(".pad")?.setPointerCapture(t.pointerId),this._calc(t))}_move(t){this._touching&&this._calc(t)}_up(){this._touching&&(this._touching=!1,this._reset())}_calc(t){const e=this.renderRoot.querySelector(".pad").getBoundingClientRect();let s=(t.clientX-e.left-e.width/2)/(e.width/2),i=(t.clientY-e.top-e.height/2)/(e.height/2);const r=Math.sqrt(s*s+i*i);r>1&&(s/=r,i/=r),this._set(s,i)}_setMode(t){this._mode=t,this._reset(),"gyro"!==t||this._gyroOk||this._startGyro(),this.requestUpdate()}_startGyro(){window.addEventListener("deviceorientation",t=>{if("gyro"!==this._mode||this._locked||null===t.beta&&null===t.gamma)return;const e=t.beta||0,s=t.gamma||0,i=screen.orientation?.angle??window.orientation??0;let r,o;90===i?(r=-s,o=e):-90===i||270===i?(r=s,o=-e):180===i?(r=-e,o=-s):(r=e,o=s),this._gyroOk||(this._gbc=r,this._ggc=o,this._gyroOk=!0),this._set((o-this._ggc)/45,(r-this._gbc)/35)})}_center(){this._reset(),"gyro"===this._mode&&(this._gyroOk=!1)}_toggleLock(){this._locked=!this._locked,this.requestUpdate()}}customElements.define("cf-yoke",mt);class gt extends lt{static properties={...lt.properties,threshold:{type:Number}};static styles=[lt.styles,o`
    :host { gap: 2px; }
    .dot {
      width: 12px; height: 12px; border-radius: 50%;
      background: var(--cf-dim); border: 1px solid var(--cf-border);
      transition: all .2s;
    }
    :host([on]) .dot {
      background: var(--color);
      box-shadow: 0 0 8px var(--color);
      border-color: var(--color);
    }
  `];static properties={...lt.properties,threshold:{type:Number},on:{type:Boolean,reflect:!0}};constructor(){super(),this.threshold=.5,this.on=!1}render(){return j`
      <div class="dot" style="--color:${this._c(this.color)}"></div>
      ${this.label?j`<span class="label">${this.label}</span>`:""}
    `}setValue(t){this.value=t,this.on=t>this.threshold}}customElements.define("cf-indicator",gt);class _t extends lt{static properties={...lt.properties,size:{type:String}};static styles=[lt.styles,o`
    :host { justify-content: center; }
    .text { font-family: 'Orbitron', sans-serif; letter-spacing: 2px; color: var(--cf-dim); }
    .text.sm { font-size: 7px; }
    .text.md { font-size: 10px; }
    .text.lg { font-size: 14px; }
  `];constructor(){super(),this.size="sm"}render(){return j`<span class="text ${this.size}">${this.label}</span>`}}customElements.define("cf-label",_t);const bt={"cf-slider":{name:"Slider",desc:"Vertical lever (throttle, mixture)"},"cf-toggle":{name:"Toggle",desc:"On/off switch"},"cf-display":{name:"Display",desc:"Numeric readout from sim"},"cf-button":{name:"Button",desc:"Press to send command"},"cf-button-group":{name:"Button Group",desc:"Select from options (magneto, flaps)"},"cf-knob":{name:"Knob",desc:"Rotary +/- (heading, course, baro)"},"cf-yoke":{name:"Yoke",desc:"2D pad + gyro (aileron/elevator)"},"cf-indicator":{name:"Indicator",desc:"Status light (gear, AP)"},"cf-label":{name:"Label",desc:"Static text or separator"}};export{lt as CfBase,pt as CfButton,ut as CfButtonGroup,dt as CfDisplay,gt as CfIndicator,ft as CfKnob,_t as CfLabel,ct as CfSlider,ht as CfToggle,mt as CfYoke,bt as WIDGETS};
